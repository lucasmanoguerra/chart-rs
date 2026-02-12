use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairGuideLineStyleBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::{Color, LineStrokeStyle, NullRenderer};

#[test]
fn chart_engine_config_defaults_crosshair_guide_line_style_behavior_to_none() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(config.crosshair_guide_line_style_behavior, None);
}

#[test]
fn chart_engine_config_applies_crosshair_guide_line_style_behavior_on_init() {
    let behavior = CrosshairGuideLineStyleBehavior {
        line_color: Color::rgb(0.14, 0.24, 0.76),
        line_width: 2.0,
        line_style: LineStrokeStyle::Dashed,
        horizontal_line_color: Some(Color::rgb(0.88, 0.27, 0.19)),
        horizontal_line_width: Some(3.0),
        horizontal_line_style: Some(LineStrokeStyle::Dotted),
        vertical_line_color: Some(Color::rgb(0.18, 0.42, 0.88)),
        vertical_line_width: Some(2.5),
        vertical_line_style: Some(LineStrokeStyle::Solid),
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_crosshair_guide_line_style_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.crosshair_guide_line_style_behavior(), behavior);
}

#[test]
fn chart_engine_config_json_without_crosshair_guide_line_style_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(config.crosshair_guide_line_style_behavior, None);
}

#[test]
fn chart_engine_config_json_parses_crosshair_guide_line_style_behavior() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "crosshair_guide_line_style_behavior": {
    "line_color": { "red": 0.1, "green": 0.2, "blue": 0.3, "alpha": 1.0 },
    "line_width": 1.5,
    "line_style": "Solid",
    "horizontal_line_color": { "red": 0.9, "green": 0.3, "blue": 0.2, "alpha": 1.0 },
    "horizontal_line_width": 3.0,
    "horizontal_line_style": "Dotted",
    "vertical_line_color": { "red": 0.2, "green": 0.4, "blue": 0.9, "alpha": 1.0 },
    "vertical_line_width": 2.0,
    "vertical_line_style": "Dashed"
  }
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.crosshair_guide_line_style_behavior,
        Some(CrosshairGuideLineStyleBehavior {
            line_color: Color::rgba(0.1, 0.2, 0.3, 1.0),
            line_width: 1.5,
            line_style: LineStrokeStyle::Solid,
            horizontal_line_color: Some(Color::rgba(0.9, 0.3, 0.2, 1.0)),
            horizontal_line_width: Some(3.0),
            horizontal_line_style: Some(LineStrokeStyle::Dotted),
            vertical_line_color: Some(Color::rgba(0.2, 0.4, 0.9, 1.0)),
            vertical_line_width: Some(2.0),
            vertical_line_style: Some(LineStrokeStyle::Dashed),
        })
    );
}
