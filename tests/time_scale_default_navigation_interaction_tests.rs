use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(10.0, 2.0),
        DataPoint::new(20.0, 3.0),
        DataPoint::new(30.0, 4.0),
        DataPoint::new(40.0, 5.0),
    ]);
    engine
}

#[test]
fn pan_with_default_navigation_keeps_span_and_shifts_window() {
    let mut engine = build_engine();
    let (before_start, before_end) = engine.time_visible_range();
    let before_span = before_end - before_start;

    engine
        .pan_time_visible_by_pixels(100.0)
        .expect("pan should work");

    let (after_start, after_end) = engine.time_visible_range();
    let after_span = after_end - after_start;

    assert!((after_span - before_span).abs() <= 1e-9);
    assert!((after_start - (before_start - 10.0)).abs() <= 1e-9);
    assert!((after_end - (before_end - 10.0)).abs() <= 1e-9);
}

#[test]
fn wheel_zoom_with_default_navigation_changes_span() {
    let mut engine = build_engine();
    let (before_start, before_end) = engine.time_visible_range();
    let before_span = before_end - before_start;

    let factor = engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("wheel zoom");
    assert!((factor - 1.2).abs() <= 1e-9);

    let (after_start, after_end) = engine.time_visible_range();
    let after_span = after_end - after_start;
    assert!(after_span < before_span);
    assert!((after_start - 8.333333333333329).abs() <= 1e-9);
    assert!((after_end - 91.66666666666667).abs() <= 1e-9);
}
