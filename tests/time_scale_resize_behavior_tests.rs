use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
    TimeScaleResizeAnchor, TimeScaleResizeBehavior,
};
use chart_rs::core::{DataPoint, OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine(width: u32) -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(width, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
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

fn prepare_navigation_engine(width: u32) -> ChartEngine<NullRenderer> {
    let mut engine = build_engine(width);
    engine.set_data(seed_points());
    engine.set_candles(seed_candles());
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: Some(20.0),
        })
        .expect("set navigation behavior");
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit to data");
    engine
}

#[test]
fn default_resize_behavior_is_disabled_and_right_anchored() {
    let engine = build_engine(1000);
    let behavior = engine.time_scale_resize_behavior();
    assert!(!behavior.lock_visible_range_on_resize);
    assert_eq!(behavior.anchor, TimeScaleResizeAnchor::Right);
}

#[test]
fn resizing_with_right_anchor_preserves_end_and_updates_span_from_spacing() {
    let mut engine = prepare_navigation_engine(1000);
    engine
        .set_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: true,
            anchor: TimeScaleResizeAnchor::Right,
        })
        .expect("set resize behavior");

    let (start_before, end_before) = engine.time_visible_range();
    assert!(((end_before - start_before) - 500.0).abs() <= 1e-9);

    engine
        .set_viewport(Viewport::new(1500, 500))
        .expect("resize should work");

    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - end_before).abs() <= 1e-9);
    assert!(((end_after - start_after) - 750.0).abs() <= 1e-9);
}

#[test]
fn resizing_with_left_anchor_preserves_start() {
    let mut engine = prepare_navigation_engine(1000);
    engine
        .set_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: true,
            anchor: TimeScaleResizeAnchor::Left,
        })
        .expect("set resize behavior");

    let (start_before, end_before) = engine.time_visible_range();

    engine
        .set_viewport(Viewport::new(1500, 500))
        .expect("resize should work");

    let (start_after, end_after) = engine.time_visible_range();
    assert!((start_after - start_before).abs() <= 1e-9);
    assert!(((end_after - start_after) - 750.0).abs() <= 1e-9);
    assert!((end_after - end_before).abs() > 1e-9);
}

#[test]
fn resizing_with_center_anchor_preserves_center() {
    let mut engine = prepare_navigation_engine(1000);
    engine
        .set_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: true,
            anchor: TimeScaleResizeAnchor::Center,
        })
        .expect("set resize behavior");

    let (start_before, end_before) = engine.time_visible_range();
    let center_before = (start_before + end_before) * 0.5;

    engine
        .set_viewport(Viewport::new(1500, 500))
        .expect("resize should work");

    let (start_after, end_after) = engine.time_visible_range();
    let center_after = (start_after + end_after) * 0.5;
    assert!((center_after - center_before).abs() <= 1e-9);
    assert!(((end_after - start_after) - 750.0).abs() <= 1e-9);
}

#[test]
fn disabling_resize_lock_keeps_visible_range_unchanged() {
    let mut engine = prepare_navigation_engine(1000);
    engine
        .set_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: false,
            anchor: TimeScaleResizeAnchor::Right,
        })
        .expect("set resize behavior");

    let (start_before, end_before) = engine.time_visible_range();
    engine
        .set_viewport(Viewport::new(1500, 500))
        .expect("resize should work");
    let (start_after, end_after) = engine.time_visible_range();

    assert!((start_after - start_before).abs() <= 1e-9);
    assert!((end_after - end_before).abs() <= 1e-9);
}

#[test]
fn resize_behavior_composes_with_fix_right_edge_clamp() {
    let mut engine = prepare_navigation_engine(1000);
    engine
        .set_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: true,
            anchor: TimeScaleResizeAnchor::Left,
        })
        .expect("set resize behavior");
    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        })
        .expect("set edge behavior");

    engine
        .set_viewport(Viewport::new(1500, 500))
        .expect("resize should work");

    let (_, full_end) = engine.time_full_range();
    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - full_end).abs() <= 1e-9);
}

#[test]
fn resize_behavior_without_spacing_preserves_logical_span() {
    let mut engine = build_engine(1000);
    engine
        .set_time_visible_range(20.0, 60.0)
        .expect("set visible range");

    engine
        .set_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: true,
            anchor: TimeScaleResizeAnchor::Right,
        })
        .expect("set resize behavior");

    engine
        .set_viewport(Viewport::new(1500, 500))
        .expect("resize should work");

    let (start_after, end_after) = engine.time_visible_range();
    assert!((start_after - 20.0).abs() <= 1e-9);
    assert!((end_after - 60.0).abs() <= 1e-9);
}
