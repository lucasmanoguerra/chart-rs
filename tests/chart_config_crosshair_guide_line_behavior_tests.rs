use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairGuideLineBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_crosshair_guide_line_behavior() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(
        config.crosshair_guide_line_behavior,
        CrosshairGuideLineBehavior::default()
    );
}

#[test]
fn chart_engine_config_applies_crosshair_guide_line_behavior_on_init() {
    let behavior = CrosshairGuideLineBehavior {
        show_lines: true,
        show_horizontal_line: false,
        show_vertical_line: true,
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_crosshair_guide_line_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.crosshair_guide_line_behavior(), behavior);
}

#[test]
fn chart_engine_config_json_without_crosshair_guide_line_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.crosshair_guide_line_behavior,
        CrosshairGuideLineBehavior::default()
    );
}

#[test]
fn chart_engine_config_json_parses_crosshair_guide_line_behavior() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "crosshair_guide_line_behavior": {
    "show_lines": false,
    "show_horizontal_line": true,
    "show_vertical_line": false
  }
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.crosshair_guide_line_behavior,
        CrosshairGuideLineBehavior {
            show_lines: false,
            show_horizontal_line: true,
            show_vertical_line: false,
        }
    );
}
