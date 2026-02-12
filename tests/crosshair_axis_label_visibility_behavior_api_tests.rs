use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairAxisLabelVisibilityBehavior};
use chart_rs::core::Viewport;
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{Color, NullRenderer, TextHAlign};

#[test]
fn crosshair_axis_label_visibility_behavior_defaults_match_render_style_defaults() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.crosshair_axis_label_visibility_behavior(),
        CrosshairAxisLabelVisibilityBehavior::default()
    );
}

#[test]
fn set_crosshair_axis_label_visibility_behavior_updates_render_style_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = CrosshairAxisLabelVisibilityBehavior {
        show_time_label: false,
        show_price_label: true,
        show_time_label_box: false,
        show_price_label_box: true,
        show_time_label_box_border: false,
        show_price_label_box_border: true,
    };
    engine
        .set_crosshair_axis_label_visibility_behavior(behavior)
        .expect("set behavior");

    assert_eq!(engine.crosshair_axis_label_visibility_behavior(), behavior);
}

#[test]
fn crosshair_axis_label_visibility_behavior_is_applied_to_rendered_labels() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_crosshair_mode(CrosshairMode::Normal);

    let style = chart_rs::api::RenderStyle {
        show_time_axis_labels: false,
        show_price_axis_labels: false,
        crosshair_time_label_color: Color::rgb(0.82, 0.30, 0.17),
        crosshair_price_label_color: Color::rgb(0.19, 0.41, 0.89),
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");
    engine
        .set_crosshair_axis_label_visibility_behavior(CrosshairAxisLabelVisibilityBehavior {
            show_time_label: false,
            show_price_label: true,
            show_time_label_box: false,
            show_price_label_box: false,
            show_time_label_box_border: false,
            show_price_label_box_border: false,
        })
        .expect("set behavior");
    engine.pointer_move(260.0, 210.0);

    let frame = engine.build_render_frame().expect("build frame");

    assert!(
        !frame
            .texts
            .iter()
            .any(|text| text.color == style.crosshair_time_label_color)
    );
    assert!(
        frame
            .texts
            .iter()
            .any(|text| text.color == style.crosshair_price_label_color
                && text.h_align == TextHAlign::Right)
    );
}
