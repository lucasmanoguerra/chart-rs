use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
};
use chart_rs::core::Viewport;
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
fn default_edge_behavior_keeps_current_unbounded_navigation() {
    let mut engine = build_engine();

    engine
        .pan_time_visible_by_pixels(200.0)
        .expect("pan left should work");
    let (start, end) = engine.time_visible_range();
    assert!((start - (-20.0)).abs() <= 1e-9);
    assert!((end - 80.0).abs() <= 1e-9);

    engine
        .pan_time_visible_by_pixels(-600.0)
        .expect("pan right should work");
    let (start, end) = engine.time_visible_range();
    assert!((start - 40.0).abs() <= 1e-9);
    assert!((end - 140.0).abs() <= 1e-9);
}

#[test]
fn fix_left_edge_blocks_panning_past_full_left_boundary_only() {
    let mut engine = build_engine();

    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: true,
            fix_right_edge: false,
        })
        .expect("set behavior");

    engine
        .pan_time_visible_by_pixels(200.0)
        .expect("pan left should clamp");
    let (start, end) = engine.time_visible_range();
    assert!(
        (start - 0.0).abs() <= 1e-9,
        "expected start=0.0, got {start}"
    );
    assert!((end - 100.0).abs() <= 1e-9, "expected end=100.0, got {end}");

    engine
        .pan_time_visible_by_pixels(-200.0)
        .expect("pan right should remain allowed");
    let (start, end) = engine.time_visible_range();
    assert!((start - 20.0).abs() <= 1e-9);
    assert!((end - 120.0).abs() <= 1e-9);
}

#[test]
fn fix_right_edge_blocks_panning_past_full_right_boundary_only() {
    let mut engine = build_engine();

    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        })
        .expect("set behavior");

    engine
        .pan_time_visible_by_pixels(-200.0)
        .expect("pan right should clamp");
    let (start, end) = engine.time_visible_range();
    assert!(
        (start - 0.0).abs() <= 1e-9,
        "expected start=0.0, got {start}"
    );
    assert!((end - 100.0).abs() <= 1e-9, "expected end=100.0, got {end}");

    engine
        .pan_time_visible_by_pixels(200.0)
        .expect("pan left should remain allowed");
    let (start, end) = engine.time_visible_range();
    assert!((start - (-20.0)).abs() <= 1e-9);
    assert!((end - 80.0).abs() <= 1e-9);
}

#[test]
fn fixing_both_edges_clamps_zoom_out_to_full_range() {
    let mut engine = build_engine();

    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: true,
            fix_right_edge: true,
        })
        .expect("set behavior");

    engine
        .zoom_time_visible_around_time(0.5, 50.0, 1e-6)
        .expect("zoom out should work");

    let (start, end) = engine.time_visible_range();
    assert!((start - 0.0).abs() <= 1e-9);
    assert!((end - 100.0).abs() <= 1e-9);
}

#[test]
fn fixing_both_edges_preserves_span_when_clamping_pan() {
    let mut engine = build_engine();

    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: true,
            fix_right_edge: true,
        })
        .expect("set behavior");

    engine
        .zoom_time_visible_around_time(2.0, 50.0, 1e-6)
        .expect("zoom in should work");
    let (start, end) = engine.time_visible_range();
    assert!((start - 25.0).abs() <= 1e-9);
    assert!((end - 75.0).abs() <= 1e-9);

    engine
        .pan_time_visible_by_pixels(-1000.0)
        .expect("pan right should clamp while preserving span");
    let (start, end) = engine.time_visible_range();
    assert!(
        (start - 50.0).abs() <= 1e-9,
        "expected start=50.0, got {start}"
    );
    assert!((end - 100.0).abs() <= 1e-9, "expected end=100.0, got {end}");

    engine
        .pan_time_visible_by_pixels(1200.0)
        .expect("pan left should clamp while preserving span");
    let (start, end) = engine.time_visible_range();
    assert!((start - 0.0).abs() <= 1e-9);
    assert!((end - 50.0).abs() <= 1e-9);
}

#[test]
fn edge_behavior_is_applied_immediately_when_policy_changes() {
    let mut engine = build_engine();

    engine
        .pan_time_visible_by_pixels(200.0)
        .expect("pan should work before constraints");
    let (start, end) = engine.time_visible_range();
    assert!((start - (-20.0)).abs() <= 1e-9);
    assert!((end - 80.0).abs() <= 1e-9);

    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: true,
            fix_right_edge: false,
        })
        .expect("set behavior should clamp current visible range");

    let (start, end) = engine.time_visible_range();
    assert!((start - 0.0).abs() <= 1e-9);
    assert!((end - 100.0).abs() <= 1e-9);
}

#[test]
fn set_time_visible_range_respects_fixed_right_edge() {
    let mut engine = build_engine();

    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        })
        .expect("set behavior");

    engine
        .set_time_visible_range(80.0, 130.0)
        .expect("set range should clamp");

    let (start, end) = engine.time_visible_range();
    assert!((start - 50.0).abs() <= 1e-9);
    assert!((end - 100.0).abs() <= 1e-9);
}
