use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairGuideLineStyleBehavior};
use chart_rs::core::Viewport;
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{Color, LineStrokeStyle, NullRenderer};

#[test]
fn crosshair_guide_line_style_behavior_defaults_match_render_style_defaults() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.crosshair_guide_line_style_behavior(),
        CrosshairGuideLineStyleBehavior::default()
    );
}

#[test]
fn set_crosshair_guide_line_style_behavior_updates_render_style_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = CrosshairGuideLineStyleBehavior {
        line_color: Color::rgb(0.16, 0.28, 0.71),
        line_width: 2.25,
        line_style: LineStrokeStyle::Dashed,
        horizontal_line_color: Some(Color::rgb(0.86, 0.30, 0.18)),
        horizontal_line_width: Some(3.0),
        horizontal_line_style: Some(LineStrokeStyle::Dotted),
        vertical_line_color: Some(Color::rgb(0.18, 0.46, 0.86)),
        vertical_line_width: Some(2.0),
        vertical_line_style: Some(LineStrokeStyle::Solid),
    };
    engine
        .set_crosshair_guide_line_style_behavior(behavior)
        .expect("set behavior");

    assert_eq!(engine.crosshair_guide_line_style_behavior(), behavior);
}

#[test]
fn crosshair_guide_line_style_behavior_is_applied_to_rendered_lines() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_crosshair_mode(CrosshairMode::Normal);

    let behavior = CrosshairGuideLineStyleBehavior {
        line_color: Color::rgb(0.10, 0.15, 0.30),
        line_width: 1.5,
        line_style: LineStrokeStyle::Solid,
        horizontal_line_color: Some(Color::rgb(0.92, 0.25, 0.17)),
        horizontal_line_width: Some(3.0),
        horizontal_line_style: Some(LineStrokeStyle::Dotted),
        vertical_line_color: Some(Color::rgb(0.20, 0.44, 0.90)),
        vertical_line_width: Some(2.0),
        vertical_line_style: Some(LineStrokeStyle::Dashed),
    };
    engine
        .set_crosshair_guide_line_style_behavior(behavior)
        .expect("set behavior");
    engine.pointer_move(260.0, 210.0);

    let frame = engine.build_render_frame().expect("build frame");
    let viewport_width = f64::from(engine.viewport().width);
    let viewport_height = f64::from(engine.viewport().height);
    let style = engine.render_style();
    let plot_right = (viewport_width - style.price_axis_width_px).clamp(0.0, viewport_width);
    let plot_bottom = (viewport_height - style.time_axis_height_px).clamp(0.0, viewport_height);
    let expected_x = 260.0_f64.clamp(0.0, plot_right);
    let expected_y = 210.0_f64.clamp(0.0, plot_bottom);

    let vertical = frame
        .lines
        .iter()
        .find(|line| {
            (line.x1 - expected_x).abs() <= 1e-9
                && (line.x2 - expected_x).abs() <= 1e-9
                && (line.y1 - 0.0).abs() <= 1e-9
                && (line.y2 - plot_bottom).abs() <= 1e-9
        })
        .expect("vertical crosshair line");
    let horizontal = frame
        .lines
        .iter()
        .find(|line| {
            (line.y1 - expected_y).abs() <= 1e-9
                && (line.y2 - expected_y).abs() <= 1e-9
                && (line.x1 - 0.0).abs() <= 1e-9
                && (line.x2 - plot_right).abs() <= 1e-9
        })
        .expect("horizontal crosshair line");

    assert_eq!(
        vertical.color,
        behavior.vertical_line_color.expect("vertical color")
    );
    assert!(
        (vertical.stroke_width - behavior.vertical_line_width.expect("vertical width")).abs()
            <= 1e-9
    );
    assert_eq!(
        vertical.stroke_style,
        behavior.vertical_line_style.expect("vertical style")
    );

    assert_eq!(
        horizontal.color,
        behavior.horizontal_line_color.expect("horizontal color")
    );
    assert!(
        (horizontal.stroke_width - behavior.horizontal_line_width.expect("horizontal width")).abs()
            <= 1e-9
    );
    assert_eq!(
        horizontal.stroke_style,
        behavior.horizontal_line_style.expect("horizontal style")
    );
}
