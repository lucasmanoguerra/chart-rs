use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairAxisLabelVisibilityBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_crosshair_axis_label_visibility_behavior() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(
        config.crosshair_axis_label_visibility_behavior,
        CrosshairAxisLabelVisibilityBehavior::default()
    );
}

#[test]
fn chart_engine_config_applies_crosshair_axis_label_visibility_behavior_on_init() {
    let behavior = CrosshairAxisLabelVisibilityBehavior {
        show_time_label: false,
        show_price_label: true,
        show_time_label_box: false,
        show_price_label_box: true,
        show_time_label_box_border: false,
        show_price_label_box_border: true,
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_crosshair_axis_label_visibility_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.crosshair_axis_label_visibility_behavior(), behavior);
}

#[test]
fn chart_engine_config_json_without_crosshair_axis_label_visibility_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.crosshair_axis_label_visibility_behavior,
        CrosshairAxisLabelVisibilityBehavior::default()
    );
}

#[test]
fn chart_engine_config_json_parses_crosshair_axis_label_visibility_behavior() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "crosshair_axis_label_visibility_behavior": {
    "show_time_label": false,
    "show_price_label": true,
    "show_time_label_box": false,
    "show_price_label_box": true,
    "show_time_label_box_border": false,
    "show_price_label_box_border": true
  }
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.crosshair_axis_label_visibility_behavior,
        CrosshairAxisLabelVisibilityBehavior {
            show_time_label: false,
            show_price_label: true,
            show_time_label_box: false,
            show_price_label_box: true,
            show_time_label_box_border: false,
            show_price_label_box_border: true,
        }
    );
}
