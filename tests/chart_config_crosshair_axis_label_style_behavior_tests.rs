use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairAxisLabelStyleBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::{Color, NullRenderer};

#[test]
fn chart_engine_config_defaults_crosshair_axis_label_style_behavior_to_none() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(config.crosshair_axis_label_style_behavior, None);
}

#[test]
fn chart_engine_config_applies_crosshair_axis_label_style_behavior_on_init() {
    let behavior = CrosshairAxisLabelStyleBehavior {
        time_label_color: Color::rgb(0.88, 0.27, 0.19),
        price_label_color: Color::rgb(0.18, 0.44, 0.88),
        time_label_font_size_px: 12.0,
        price_label_font_size_px: 13.0,
        time_label_offset_y_px: 6.0,
        price_label_offset_y_px: 10.0,
        time_label_padding_x_px: 20.0,
        price_label_padding_right_px: 12.0,
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_crosshair_axis_label_style_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.crosshair_axis_label_style_behavior(), behavior);
}

#[test]
fn chart_engine_config_json_without_crosshair_axis_label_style_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(config.crosshair_axis_label_style_behavior, None);
}

#[test]
fn chart_engine_config_json_parses_crosshair_axis_label_style_behavior() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "crosshair_axis_label_style_behavior": {
    "time_label_color": { "red": 0.9, "green": 0.3, "blue": 0.2, "alpha": 1.0 },
    "price_label_color": { "red": 0.2, "green": 0.4, "blue": 0.9, "alpha": 1.0 },
    "time_label_font_size_px": 12.0,
    "price_label_font_size_px": 13.0,
    "time_label_offset_y_px": 6.0,
    "price_label_offset_y_px": 10.0,
    "time_label_padding_x_px": 20.0,
    "price_label_padding_right_px": 12.0
  }
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.crosshair_axis_label_style_behavior,
        Some(CrosshairAxisLabelStyleBehavior {
            time_label_color: Color::rgba(0.9, 0.3, 0.2, 1.0),
            price_label_color: Color::rgba(0.2, 0.4, 0.9, 1.0),
            time_label_font_size_px: 12.0,
            price_label_font_size_px: 13.0,
            time_label_offset_y_px: 6.0,
            price_label_offset_y_px: 10.0,
            time_label_padding_x_px: 20.0,
            price_label_padding_right_px: 12.0,
        })
    );
}
