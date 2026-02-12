use chart_rs::ChartError;
use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior, TimeScaleRealtimeAppendBehavior,
};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("disable default spacing navigation");
    engine
}

#[test]
fn update_point_on_empty_series_appends() {
    let mut engine = build_engine();
    engine
        .update_point(DataPoint::new(110.0, 10.0))
        .expect("update point");

    assert_eq!(engine.points().len(), 1);
    let (_, full_end) = engine.time_full_range();
    assert!((full_end - 110.0).abs() <= 1e-9);
}

#[test]
fn update_point_same_time_replaces_latest_sample() {
    let mut engine = build_engine();
    engine.set_data(vec![DataPoint::new(10.0, 1.0), DataPoint::new(20.0, 2.0)]);
    let before = engine.time_visible_range();

    engine
        .update_point(DataPoint::new(20.0, 99.0))
        .expect("replace point");

    assert_eq!(engine.points().len(), 2);
    assert!((engine.points()[1].y - 99.0).abs() <= 1e-9);
    assert_eq!(engine.time_visible_range(), before);
}

#[test]
fn update_point_rejects_out_of_order_time() {
    let mut engine = build_engine();
    engine.set_data(vec![DataPoint::new(10.0, 1.0), DataPoint::new(20.0, 2.0)]);

    let err = engine
        .update_point(DataPoint::new(19.0, 3.0))
        .expect_err("older time must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn update_point_newer_time_appends_and_applies_realtime_follow() {
    let mut engine = build_engine();
    engine.set_data(vec![DataPoint::new(10.0, 1.0), DataPoint::new(20.0, 2.0)]);

    engine
        .update_point(DataPoint::new(110.0, 3.0))
        .expect("newer point append");

    assert_eq!(engine.points().len(), 3);
    let (start, end) = engine.time_visible_range();
    assert!((start - 10.0).abs() <= 1e-9);
    assert!((end - 110.0).abs() <= 1e-9);
}

#[test]
fn update_point_newer_time_can_disable_realtime_follow() {
    let mut engine = build_engine();
    engine
        .set_time_scale_realtime_append_behavior(TimeScaleRealtimeAppendBehavior {
            preserve_right_edge_on_append: false,
            right_edge_tolerance_bars: 0.75,
        })
        .expect("set realtime behavior");
    engine.set_data(vec![DataPoint::new(10.0, 1.0), DataPoint::new(20.0, 2.0)]);

    engine
        .update_point(DataPoint::new(110.0, 3.0))
        .expect("newer point append");

    let (start, end) = engine.time_visible_range();
    assert!((start - 0.0).abs() <= 1e-9);
    assert!((end - 100.0).abs() <= 1e-9);
}

#[test]
fn update_candle_supports_replace_and_order_validation() {
    let mut engine = build_engine();
    let c10 = OhlcBar::new(10.0, 1.0, 2.0, 0.5, 1.5).expect("valid candle");
    let c20 = OhlcBar::new(20.0, 2.0, 3.0, 1.5, 2.5).expect("valid candle");
    let c20_replace = OhlcBar::new(20.0, 2.1, 3.1, 1.6, 2.6).expect("valid candle");
    let c19 = OhlcBar::new(19.0, 2.0, 3.0, 1.5, 2.5).expect("valid candle");

    engine.set_candles(vec![c10, c20]);
    engine.update_candle(c20_replace).expect("replace candle");
    assert_eq!(engine.candles().len(), 2);
    assert!((engine.candles()[1].close - 2.6).abs() <= 1e-9);

    let err = engine
        .update_candle(c19)
        .expect_err("older candle time must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
