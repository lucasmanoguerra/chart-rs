use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn build_render_frame_includes_series_and_axis_primitives() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(10.0, 10.0),
        DataPoint::new(20.0, 25.0),
        DataPoint::new(40.0, 15.0),
    ]);

    let frame = engine.build_render_frame().expect("build frame");
    frame.validate().expect("valid frame");

    assert!(frame.lines.len() >= 14, "expected series + axis lines");
    assert_eq!(frame.texts.len(), 10, "expected 5 time + 5 price labels");
}

#[test]
fn null_renderer_receives_computed_frame_counts() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 450), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(5.0, 10.0),
        DataPoint::new(15.0, 20.0),
        DataPoint::new(30.0, 15.0),
    ]);

    engine.render().expect("render");
    let renderer = engine.into_renderer();

    assert!(renderer.last_line_count >= 14);
    assert_eq!(renderer.last_text_count, 10);
}
