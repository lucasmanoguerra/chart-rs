use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

fn build_engine(time_start: f64, time_end: f64) -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), time_start, time_end)
        .with_price_domain(0.0, 1.0);
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
fn pan_time_visible_by_pixels_translates_visible_range() {
    let mut engine = build_engine(0.0, 100.0);

    engine
        .pan_time_visible_by_pixels(100.0)
        .expect("pan by pixel should work");
    let (start, end) = engine.time_visible_range();
    assert!((start - (-10.0)).abs() <= 1e-9);
    assert!((end - 90.0).abs() <= 1e-9);

    engine
        .pan_time_visible_by_pixels(-100.0)
        .expect("pan by pixel should work");
    let (start, end) = engine.time_visible_range();
    assert!((start - 0.0).abs() <= 1e-9);
    assert!((end - 100.0).abs() <= 1e-9);
}

#[test]
fn zoom_time_visible_around_pixel_keeps_anchor_stable() {
    let mut engine = build_engine(0.0, 100.0);

    let anchor_px = 250.0;
    let anchor_time_before = engine.map_pixel_to_x(anchor_px).expect("anchor time");

    engine
        .zoom_time_visible_around_pixel(2.0, anchor_px, 1e-6)
        .expect("zoom should work");

    let (start, end) = engine.time_visible_range();
    assert!((start - 12.5).abs() <= 1e-9);
    assert!((end - 62.5).abs() <= 1e-9);

    let anchor_time_after = engine.map_pixel_to_x(anchor_px).expect("anchor time after");
    assert!((anchor_time_after - anchor_time_before).abs() <= 1e-9);
}

#[test]
fn zoom_time_visible_respects_min_span() {
    let mut engine = build_engine(0.0, 10.0);

    engine
        .zoom_time_visible_around_time(10_000.0, 5.0, 4.0)
        .expect("zoom should clamp");

    let (start, end) = engine.time_visible_range();
    assert!(((end - start) - 4.0).abs() <= 1e-9);
}

#[test]
fn zoom_time_visible_rejects_invalid_factor() {
    let mut engine = build_engine(0.0, 10.0);

    let err = engine
        .zoom_time_visible_around_time(0.0, 5.0, 1.0)
        .expect_err("zero factor must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
