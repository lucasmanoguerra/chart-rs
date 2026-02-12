use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
};
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

fn prepare_engine() -> ChartEngine<NullRenderer> {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine.set_candles(seed_candles());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit to data");
    engine
}

#[test]
fn scroll_to_realtime_without_navigation_preserves_span_and_aligns_right_edge() {
    let mut engine = prepare_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("disable default spacing navigation");
    engine
        .set_time_visible_range(-50.0, 30.0)
        .expect("set visible range");

    let changed = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should succeed");
    assert!(changed);

    let (_, full_end) = engine.time_full_range();
    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - full_end).abs() <= 1e-9);
    assert!(((end_after - start_after) - 80.0).abs() <= 1e-9);
}

#[test]
fn scroll_to_realtime_with_right_offset_applies_navigation_target() {
    let mut engine = prepare_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .expect("set navigation behavior");

    let changed = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should succeed");
    assert!(!changed);

    let (_, full_end) = engine.time_full_range();
    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - (full_end + 20.0)).abs() <= 1e-9);
}

#[test]
fn scroll_to_realtime_with_spacing_applies_expected_span() {
    let mut engine = prepare_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: Some(20.0),
        })
        .expect("set navigation behavior");

    let changed = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should succeed");
    assert!(!changed);

    let (_, full_end) = engine.time_full_range();
    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - full_end).abs() <= 1e-9);
    assert!(((end_after - start_after) - 500.0).abs() <= 1e-9);
}

#[test]
fn scroll_to_realtime_applies_right_offset_pixels_policy() {
    let mut engine = prepare_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("disable default spacing navigation");
    let (start_before, end_before) = engine.time_visible_range();
    let visible_span_before = end_before - start_before;
    engine
        .set_time_scale_right_offset_px(Some(100.0))
        .expect("set right offset px");

    let changed = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should succeed");
    assert!(!changed);

    let (_, full_end) = engine.time_full_range();
    let (_, end_after) = engine.time_visible_range();
    let expected_offset = (visible_span_before / 1000.0) * 100.0;
    assert!((end_after - (full_end + expected_offset)).abs() <= 1e-9);
}

#[test]
fn scroll_to_realtime_respects_fix_right_edge_constraint() {
    let mut engine = prepare_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 3.0,
            bar_spacing_px: None,
        })
        .expect("set navigation behavior");
    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        })
        .expect("set edge behavior");
    engine
        .set_time_visible_range(-50.0, 30.0)
        .expect("set visible range");

    let changed = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should succeed");
    assert!(changed);

    let (_, full_end) = engine.time_full_range();
    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - full_end).abs() <= 1e-9);
}

#[test]
fn scroll_to_realtime_is_idempotent_when_already_at_tail() {
    let mut engine = prepare_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("disable default spacing navigation");
    engine
        .set_time_visible_range(-50.0, 30.0)
        .expect("set visible range away from realtime");
    let first = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should succeed");
    assert!(first);
    let changed = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should succeed");
    assert!(!changed);
}
