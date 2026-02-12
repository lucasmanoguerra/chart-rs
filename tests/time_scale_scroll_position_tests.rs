use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig, TimeScaleEdgeBehavior};
use chart_rs::core::{DataPoint, OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

fn seed_points() -> Vec<DataPoint> {
    (0..10)
        .map(|index| DataPoint::new(index as f64 * 10.0, 100.0 + index as f64))
        .collect()
}

fn seed_candles() -> Vec<OhlcBar> {
    seed_points()
        .into_iter()
        .map(|point| OhlcBar::new(point.x, point.y, point.y + 1.0, point.y - 1.0, point.y))
        .collect::<Result<Vec<_>, _>>()
        .expect("valid candles")
}

fn prepare_fitted_engine() -> ChartEngine<NullRenderer> {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine.set_candles(seed_candles());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit to data");
    engine
}

#[test]
fn scroll_position_is_zero_when_right_edge_matches_realtime() {
    let mut engine = prepare_fitted_engine();
    engine.reset_time_visible_range();
    let position = engine
        .time_scroll_position_bars()
        .expect("step should be resolvable");
    assert!(position.abs() <= 1e-9);
}

#[test]
fn scroll_position_reports_positive_and_negative_offsets() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_visible_range(10.0, 110.0)
        .expect("set visible range");
    let positive = engine
        .time_scroll_position_bars()
        .expect("step should be resolvable");
    assert!((positive - 2.0).abs() <= 1e-9);

    engine
        .set_time_visible_range(-20.0, 80.0)
        .expect("set visible range");
    let negative = engine
        .time_scroll_position_bars()
        .expect("step should be resolvable");
    assert!((negative - (-1.0)).abs() <= 1e-9);
}

#[test]
fn scroll_to_position_preserves_span_and_applies_target_offset() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_visible_range(-20.0, 80.0)
        .expect("set visible range");

    let changed = engine
        .scroll_time_to_position_bars(3.0)
        .expect("scroll should succeed");
    assert!(changed);

    let (start, end) = engine.time_visible_range();
    assert!(((end - start) - 100.0).abs() <= 1e-9);

    let position = engine
        .time_scroll_position_bars()
        .expect("step should be resolvable");
    assert!((position - 3.0).abs() <= 1e-9);
}

#[test]
fn scroll_to_position_respects_fixed_right_edge_constraint() {
    let mut engine = prepare_fitted_engine();
    engine.reset_time_visible_range();
    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        })
        .expect("set edge behavior");

    let changed = engine
        .scroll_time_to_position_bars(2.0)
        .expect("scroll should succeed");
    assert!(changed);

    let position = engine
        .time_scroll_position_bars()
        .expect("step should be resolvable");
    assert!(position.abs() <= 1e-9);
}

#[test]
fn scroll_to_position_rejects_invalid_input() {
    let mut engine = prepare_fitted_engine();
    let err = engine
        .scroll_time_to_position_bars(f64::NAN)
        .expect_err("nan must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn scroll_to_non_zero_position_requires_reference_step() {
    let mut engine = build_engine();
    let err = engine
        .scroll_time_to_position_bars(1.0)
        .expect_err("no data step should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
