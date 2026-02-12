use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior, TimeScaleScrollZoomBehavior,
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
fn default_scroll_zoom_behavior_keeps_right_bar_stays_disabled() {
    let engine = build_engine();
    let behavior = engine.time_scale_scroll_zoom_behavior();
    assert!(!behavior.right_bar_stays_on_scroll);
}

#[test]
fn wheel_zoom_without_right_bar_stays_moves_right_edge() {
    let mut engine = build_engine();
    let (_, end_before) = engine.time_visible_range();

    engine
        .wheel_zoom_time_visible(-120.0, 250.0, 0.2, 1e-6)
        .expect("wheel zoom");

    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - end_before).abs() > 1e-9);
}

#[test]
fn wheel_zoom_with_right_bar_stays_preserves_right_edge() {
    let mut engine = build_engine();
    engine
        .set_time_scale_scroll_zoom_behavior(TimeScaleScrollZoomBehavior {
            right_bar_stays_on_scroll: true,
        })
        .expect("set scroll zoom behavior");

    let (_, end_before) = engine.time_visible_range();
    engine
        .wheel_zoom_time_visible(-120.0, 250.0, 0.2, 1e-6)
        .expect("wheel zoom");
    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - end_before).abs() <= 1e-9);
}

#[test]
fn pinch_zoom_with_right_bar_stays_preserves_right_edge() {
    let mut engine = build_engine();
    engine
        .set_time_scale_scroll_zoom_behavior(TimeScaleScrollZoomBehavior {
            right_bar_stays_on_scroll: true,
        })
        .expect("set scroll zoom behavior");

    let (_, end_before) = engine.time_visible_range();
    engine
        .pinch_zoom_time_visible(1.2, 300.0, 1e-6)
        .expect("pinch zoom");
    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - end_before).abs() <= 1e-9);
}

#[test]
fn switching_behavior_runtime_changes_zoom_anchor_policy() {
    let mut engine = build_engine();
    engine
        .wheel_zoom_time_visible(-120.0, 250.0, 0.2, 1e-6)
        .expect("wheel zoom");
    let (_, end_after_default) = engine.time_visible_range();

    engine.reset_time_visible_range();
    engine
        .set_time_scale_scroll_zoom_behavior(TimeScaleScrollZoomBehavior {
            right_bar_stays_on_scroll: true,
        })
        .expect("set scroll zoom behavior");
    let (_, end_before) = engine.time_visible_range();
    engine
        .wheel_zoom_time_visible(-120.0, 250.0, 0.2, 1e-6)
        .expect("wheel zoom");
    let (_, end_after_anchor_right) = engine.time_visible_range();

    assert!((end_after_default - end_before).abs() > 1e-9);
    assert!((end_after_anchor_right - end_before).abs() <= 1e-9);
}
