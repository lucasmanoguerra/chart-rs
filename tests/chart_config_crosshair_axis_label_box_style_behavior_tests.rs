use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairAxisLabelBoxStyleBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::{Color, NullRenderer};

#[test]
fn chart_engine_config_defaults_crosshair_axis_label_box_style_behavior_to_none() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(config.crosshair_axis_label_box_style_behavior, None);
}

#[test]
fn chart_engine_config_applies_crosshair_axis_label_box_style_behavior_on_init() {
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
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_crosshair_axis_label_box_style_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.crosshair_axis_label_box_style_behavior(), behavior);
}

#[test]
fn chart_engine_config_json_without_crosshair_axis_label_box_style_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(config.crosshair_axis_label_box_style_behavior, None);
}

#[test]
fn chart_engine_config_json_parses_crosshair_axis_label_box_style_behavior() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "crosshair_axis_label_box_style_behavior": {
    "box_color": { "red": 0.93, "green": 0.85, "blue": 0.23, "alpha": 1.0 },
    "time_box_color": { "red": 0.9, "green": 0.34, "blue": 0.24, "alpha": 1.0 },
    "price_box_color": { "red": 0.22, "green": 0.44, "blue": 0.9, "alpha": 1.0 },
    "box_border_color": { "red": 0.3, "green": 0.3, "blue": 0.3, "alpha": 1.0 },
    "time_box_border_color": { "red": 0.76, "green": 0.22, "blue": 0.17, "alpha": 1.0 },
    "price_box_border_color": { "red": 0.16, "green": 0.35, "blue": 0.72, "alpha": 1.0 },
    "box_border_width_px": 1.0,
    "time_box_border_width_px": 2.0,
    "price_box_border_width_px": 3.0,
    "box_corner_radius_px": 2.0,
    "time_box_corner_radius_px": 4.0,
    "price_box_corner_radius_px": 5.0
  }
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.crosshair_axis_label_box_style_behavior,
        Some(CrosshairAxisLabelBoxStyleBehavior {
            box_color: Color::rgba(0.93, 0.85, 0.23, 1.0),
            time_box_color: Some(Color::rgba(0.9, 0.34, 0.24, 1.0)),
            price_box_color: Some(Color::rgba(0.22, 0.44, 0.9, 1.0)),
            box_border_color: Color::rgba(0.3, 0.3, 0.3, 1.0),
            time_box_border_color: Color::rgba(0.76, 0.22, 0.17, 1.0),
            price_box_border_color: Color::rgba(0.16, 0.35, 0.72, 1.0),
            box_border_width_px: 1.0,
            time_box_border_width_px: 2.0,
            price_box_border_width_px: 3.0,
            box_corner_radius_px: 2.0,
            time_box_corner_radius_px: 4.0,
            price_box_corner_radius_px: 5.0,
        })
    );
}
