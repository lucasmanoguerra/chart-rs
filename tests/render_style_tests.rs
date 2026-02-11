use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig, RenderStyle};
use chart_rs::core::Viewport;
use chart_rs::render::{Color, NullRenderer};

#[test]
fn default_render_style_produces_grid_and_axis_lines() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    let style = engine.render_style();
    let frame = engine.build_render_frame().expect("frame");

    let grid_lines = frame
        .lines
        .iter()
        .filter(|line| line.color == style.grid_line_color)
        .count();
    let axis_lines = frame
        .lines
        .iter()
        .filter(|line| line.color == style.axis_border_color)
        .count();

    assert!(grid_lines >= 4);
    assert!(axis_lines >= 4);
}

#[test]
fn custom_render_style_is_applied_to_frame() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let custom_style = RenderStyle {
        series_line_color: Color::rgb(0.9, 0.2, 0.2),
        grid_line_color: Color::rgb(0.1, 0.7, 0.4),
        axis_border_color: Color::rgb(0.2, 0.2, 0.2),
        axis_label_color: Color::rgb(0.0, 0.0, 0.0),
        grid_line_width: 2.0,
        axis_line_width: 1.5,
        price_axis_width_px: 84.0,
        time_axis_height_px: 28.0,
    };
    engine
        .set_render_style(custom_style)
        .expect("set render style");

    let frame = engine.build_render_frame().expect("frame");
    assert!(
        frame
            .lines
            .iter()
            .any(|line| line.color == custom_style.grid_line_color && line.stroke_width == 2.0)
    );
    assert!(
        frame
            .lines
            .iter()
            .any(|line| line.color == custom_style.axis_border_color && line.stroke_width == 1.5)
    );
}

#[test]
fn invalid_render_style_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.grid_line_width = 0.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
