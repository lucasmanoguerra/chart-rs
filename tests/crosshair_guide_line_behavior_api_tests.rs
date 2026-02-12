use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairGuideLineBehavior};
use chart_rs::core::Viewport;
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{NullRenderer, TextHAlign};

#[test]
fn crosshair_guide_line_behavior_defaults_match_render_style_defaults() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.crosshair_guide_line_behavior(),
        CrosshairGuideLineBehavior::default()
    );
}

#[test]
fn set_crosshair_guide_line_behavior_updates_render_style_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = CrosshairGuideLineBehavior {
        show_lines: true,
        show_horizontal_line: false,
        show_vertical_line: true,
    };
    engine
        .set_crosshair_guide_line_behavior(behavior)
        .expect("set behavior");

    assert_eq!(engine.crosshair_guide_line_behavior(), behavior);
    let style = engine.render_style();
    assert!(style.show_crosshair_lines);
    assert!(!style.show_crosshair_horizontal_line);
    assert!(style.show_crosshair_vertical_line);
}

#[test]
fn crosshair_guide_line_behavior_shared_toggle_hides_lines_but_keeps_labels() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine.pointer_move(240.0, 180.0);
    let mut style = engine.render_style();
    style.show_price_axis_labels = false;
    style.show_time_axis_labels = false;
    engine.set_render_style(style).expect("set style");
    engine
        .set_crosshair_guide_line_behavior(CrosshairGuideLineBehavior {
            show_lines: false,
            show_horizontal_line: true,
            show_vertical_line: true,
        })
        .expect("set behavior");

    let style = engine.render_style();
    let frame = engine.build_render_frame().expect("build frame");

    assert!(!frame.lines.iter().any(|line| {
        line.color == style.crosshair_line_color
            && (line.stroke_width - style.crosshair_line_width).abs() <= 1e-9
    }));
    assert!(
        frame
            .texts
            .iter()
            .any(|text| text.h_align == TextHAlign::Center)
    );
    assert!(
        frame
            .texts
            .iter()
            .any(|text| text.h_align == TextHAlign::Right)
    );
}
