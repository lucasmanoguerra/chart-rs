use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairAxisLabelStyleBehavior, RenderStyle};
use chart_rs::core::Viewport;
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{Color, NullRenderer, TextHAlign};

#[test]
fn crosshair_axis_label_style_behavior_defaults_match_render_style_defaults() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.crosshair_axis_label_style_behavior(),
        CrosshairAxisLabelStyleBehavior::default()
    );
}

#[test]
fn set_crosshair_axis_label_style_behavior_updates_render_style_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = CrosshairAxisLabelStyleBehavior {
        time_label_color: Color::rgb(0.88, 0.27, 0.18),
        price_label_color: Color::rgb(0.17, 0.42, 0.89),
        time_label_font_size_px: 13.0,
        price_label_font_size_px: 14.0,
        time_label_offset_y_px: 7.0,
        price_label_offset_y_px: 9.0,
        time_label_padding_x_px: 18.0,
        price_label_padding_right_px: 12.0,
    };
    engine
        .set_crosshair_axis_label_style_behavior(behavior)
        .expect("set behavior");

    assert_eq!(engine.crosshair_axis_label_style_behavior(), behavior);
}

#[test]
fn crosshair_axis_label_style_behavior_is_applied_to_rendered_labels() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine
        .set_render_style(RenderStyle {
            show_time_axis_labels: false,
            show_price_axis_labels: false,
            show_crosshair_time_label_box: false,
            show_crosshair_price_label_box: false,
            ..engine.render_style()
        })
        .expect("set style");

    let behavior = CrosshairAxisLabelStyleBehavior {
        time_label_color: Color::rgb(0.86, 0.29, 0.19),
        price_label_color: Color::rgb(0.18, 0.44, 0.88),
        time_label_font_size_px: 12.5,
        price_label_font_size_px: 13.5,
        time_label_offset_y_px: 6.0,
        price_label_offset_y_px: 11.0,
        time_label_padding_x_px: 25.0,
        price_label_padding_right_px: 14.0,
    };
    engine
        .set_crosshair_axis_label_style_behavior(behavior)
        .expect("set behavior");
    engine.pointer_move(2.0, 210.0);

    let frame = engine.build_render_frame().expect("build frame");
    let style = engine.crosshair_axis_label_style_behavior();
    let render_style = engine.render_style();
    let viewport_width = f64::from(engine.viewport().width);
    let viewport_height = f64::from(engine.viewport().height);
    let plot_right = (viewport_width - render_style.price_axis_width_px).clamp(0.0, viewport_width);
    let plot_bottom =
        (viewport_height - render_style.time_axis_height_px).clamp(0.0, viewport_height);
    let crosshair_x = 2.0_f64.clamp(0.0, plot_right);
    let crosshair_y = 210.0_f64.clamp(0.0, plot_bottom);

    let expected_time_x = crosshair_x.clamp(
        style.time_label_padding_x_px,
        (plot_right - style.time_label_padding_x_px).max(style.time_label_padding_x_px),
    );
    let expected_time_y = (plot_bottom + style.time_label_offset_y_px)
        .min((viewport_height - style.time_label_font_size_px).max(0.0));
    let expected_price_x =
        (viewport_width - style.price_label_padding_right_px).clamp(plot_right, viewport_width);
    let expected_price_y = (crosshair_y - style.price_label_offset_y_px).max(0.0);

    let time_label = frame
        .texts
        .iter()
        .find(|text| text.color == style.time_label_color && text.h_align == TextHAlign::Center)
        .expect("time label");
    assert!((time_label.font_size_px - style.time_label_font_size_px).abs() <= 1e-9);
    assert!((time_label.x - expected_time_x).abs() <= 1e-9);
    assert!((time_label.y - expected_time_y).abs() <= 1e-9);

    let price_label = frame
        .texts
        .iter()
        .find(|text| text.color == style.price_label_color && text.h_align == TextHAlign::Right)
        .expect("price label");
    assert!((price_label.font_size_px - style.price_label_font_size_px).abs() <= 1e-9);
    assert!((price_label.x - expected_price_x).abs() <= 1e-9);
    assert!((price_label.y - expected_price_y).abs() <= 1e-9);
}
