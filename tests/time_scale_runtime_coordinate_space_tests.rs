use chart_rs::api::{ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior};
use chart_rs::core::{DataPoint, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

fn seed_points() -> Vec<DataPoint> {
    (0..=20)
        .map(|index| DataPoint::new(index as f64 * 10.0, 100.0 + index as f64))
        .collect()
}

#[test]
fn zoom_time_visible_around_pixel_preserves_anchor_time() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("navigation");

    let anchor_px = 350.0;
    let anchor_time_before = engine.map_pixel_to_x(anchor_px).expect("anchor before");
    engine
        .zoom_time_visible_around_pixel(1.35, anchor_px, 1e-6)
        .expect("zoom");
    let anchor_time_after = engine.map_pixel_to_x(anchor_px).expect("anchor after");

    assert!((anchor_time_after - anchor_time_before).abs() <= 1e-9);
}

#[test]
fn pan_time_visible_by_pixels_updates_right_offset_by_bar_delta() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("navigation");

    let (_, full_end) = engine.time_full_range();
    let (start_before, end_before) = engine.time_visible_range();
    let span_before = end_before - start_before;
    let step = 10.0;
    let right_offset_before = (end_before - full_end) / step;
    let bar_spacing_before = f64::from(engine.viewport().width) / (span_before / step);

    engine
        .pan_time_visible_by_pixels(120.0)
        .expect("pan by pixels");

    let (_, end_after) = engine.time_visible_range();
    let right_offset_after = (end_after - full_end) / step;
    let expected_after = right_offset_before - 120.0 / bar_spacing_before;
    assert!((right_offset_after - expected_after).abs() <= 1e-9);
}
