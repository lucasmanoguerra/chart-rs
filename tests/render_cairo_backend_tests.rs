#![cfg(feature = "cairo-backend")]

use cairo::{Context, Format, ImageSurface};
use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::CairoRenderer;

#[test]
fn cairo_renderer_rejects_invalid_surface_size() {
    let err = CairoRenderer::new(0, 480).expect_err("invalid width must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn cairo_renderer_renders_series_and_axis_primitives() {
    let renderer = CairoRenderer::new(900, 500).expect("renderer");
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(10.0, 10.0),
        DataPoint::new(20.0, 20.0),
        DataPoint::new(40.0, 15.0),
    ]);

    engine.render().expect("render");
    let renderer = engine.into_renderer();
    let stats = renderer.last_stats();

    assert!(stats.lines_drawn >= 14);
    assert_eq!(stats.texts_drawn, 10);
}

#[test]
fn cairo_renderer_can_draw_on_external_context() {
    let renderer = CairoRenderer::new(600, 320).expect("renderer");
    let config =
        ChartEngineConfig::new(Viewport::new(600, 320), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(30.0, 20.0),
        DataPoint::new(60.0, 15.0),
    ]);

    let surface = ImageSurface::create(Format::ARgb32, 600, 320).expect("surface");
    let context = Context::new(&surface).expect("context");
    engine
        .render_on_cairo_context(&context)
        .expect("render on context");

    let renderer = engine.into_renderer();
    assert!(renderer.last_stats().lines_drawn >= 14);
}
