use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairAxisLabelBoxStyleBehavior, RenderStyle,
};
use chart_rs::core::Viewport;
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{Color, NullRenderer};

#[test]
fn crosshair_axis_label_box_style_behavior_defaults_match_render_style_defaults() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.crosshair_axis_label_box_style_behavior(),
        CrosshairAxisLabelBoxStyleBehavior::default()
    );
}

#[test]
fn set_crosshair_axis_label_box_style_behavior_updates_render_style_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = CrosshairAxisLabelBoxStyleBehavior {
        box_color: Color::rgb(0.93, 0.85, 0.23),
        time_box_color: Some(Color::rgb(0.90, 0.34, 0.24)),
        price_box_color: Some(Color::rgb(0.22, 0.44, 0.90)),
        box_border_color: Color::rgb(0.30, 0.30, 0.30),
        time_box_border_color: Color::rgb(0.76, 0.22, 0.17),
        price_box_border_color: Color::rgb(0.16, 0.35, 0.72),
        box_border_width_px: 1.0,
        time_box_border_width_px: 2.0,
        price_box_border_width_px: 3.0,
        box_corner_radius_px: 2.0,
        time_box_corner_radius_px: 4.0,
        price_box_corner_radius_px: 5.0,
    };
    engine
        .set_crosshair_axis_label_box_style_behavior(behavior)
        .expect("set behavior");

    assert_eq!(engine.crosshair_axis_label_box_style_behavior(), behavior);
}

#[test]
fn crosshair_axis_label_box_style_behavior_is_applied_to_rendered_rects() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine
        .set_render_style(RenderStyle {
            show_time_axis_labels: false,
            show_price_axis_labels: false,
            show_crosshair_time_label: true,
            show_crosshair_price_label: true,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            show_crosshair_time_label_box_border: true,
            show_crosshair_price_label_box_border: true,
            ..engine.render_style()
        })
        .expect("set style");

    let behavior = CrosshairAxisLabelBoxStyleBehavior {
        box_color: Color::rgb(0.93, 0.85, 0.23),
        time_box_color: Some(Color::rgb(0.90, 0.34, 0.24)),
        price_box_color: Some(Color::rgb(0.22, 0.44, 0.90)),
        box_border_color: Color::rgb(0.30, 0.30, 0.30),
        time_box_border_color: Color::rgb(0.76, 0.22, 0.17),
        price_box_border_color: Color::rgb(0.16, 0.35, 0.72),
        box_border_width_px: 1.0,
        time_box_border_width_px: 2.0,
        price_box_border_width_px: 3.0,
        box_corner_radius_px: 1.0,
        time_box_corner_radius_px: 4.0,
        price_box_corner_radius_px: 5.0,
    };
    engine
        .set_crosshair_axis_label_box_style_behavior(behavior)
        .expect("set behavior");
    engine.pointer_move(260.0, 210.0);

    let frame = engine.build_render_frame().expect("build frame");

    assert!(frame.rects.iter().any(|rect| {
        rect.fill_color == behavior.time_box_color.expect("time box color")
            && (rect.border_width - behavior.time_box_border_width_px).abs() <= 1e-9
            && rect.border_color == behavior.time_box_border_color
    }));
    assert!(frame.rects.iter().any(|rect| {
        rect.fill_color == behavior.price_box_color.expect("price box color")
            && (rect.border_width - behavior.price_box_border_width_px).abs() <= 1e-9
            && rect.border_color == behavior.price_box_border_color
    }));
}
