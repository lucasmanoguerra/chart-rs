use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior};
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
fn wheel_zoom_negative_delta_zooms_in_and_keeps_anchor_stable() {
    let mut engine = build_engine();

    let anchor_px = 250.0;
    let anchor_time_before = engine.map_pixel_to_x(anchor_px).expect("anchor time");
    let (start_before, end_before) = engine.time_visible_range();
    let span_before = end_before - start_before;

    let factor = engine
        .wheel_zoom_time_visible(-120.0, anchor_px, 0.2, 1e-6)
        .expect("wheel zoom");
    assert!((factor - 1.2).abs() <= 1e-9);

    let (start_after, end_after) = engine.time_visible_range();
    let span_after = end_after - start_after;
    assert!(span_after < span_before);

    let anchor_time_after = engine.map_pixel_to_x(anchor_px).expect("anchor time after");
    assert!((anchor_time_after - anchor_time_before).abs() <= 1e-9);
}

#[test]
fn wheel_zoom_positive_delta_zooms_out() {
    let mut engine = build_engine();

    let (start_before, end_before) = engine.time_visible_range();
    let span_before = end_before - start_before;

    let factor = engine
        .wheel_zoom_time_visible(120.0, 250.0, 0.2, 1e-6)
        .expect("wheel zoom");
    assert!((factor - (1.0 / 1.2)).abs() <= 1e-9);

    let (start_after, end_after) = engine.time_visible_range();
    let span_after = end_after - start_after;
    assert!(span_after > span_before);
}

#[test]
fn wheel_zoom_zero_delta_is_noop() {
    let mut engine = build_engine();

    let before = engine.time_visible_range();
    let factor = engine
        .wheel_zoom_time_visible(0.0, 500.0, 0.2, 1e-6)
        .expect("wheel zoom noop");
    let after = engine.time_visible_range();

    assert!((factor - 1.0).abs() <= 1e-12);
    assert_eq!(before, after);
}

#[test]
fn wheel_zoom_rejects_invalid_inputs() {
    let mut engine = build_engine();

    let err = engine
        .wheel_zoom_time_visible(f64::NAN, 100.0, 0.2, 1e-6)
        .expect_err("nan delta must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));

    let err = engine
        .wheel_zoom_time_visible(-120.0, 100.0, 0.0, 1e-6)
        .expect_err("invalid step ratio must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
