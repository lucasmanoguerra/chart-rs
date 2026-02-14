use chart_rs::api::{
    CandlestickBodyMode, CandlestickStyleBehavior, ChartEngine, ChartEngineConfig,
};
use chart_rs::core::Viewport;
use chart_rs::render::{Color, NullRenderer};

#[test]
fn chart_engine_config_defaults_candlestick_style_behavior_to_none() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(config.candlestick_style_behavior, None);
}

#[test]
fn chart_engine_config_applies_candlestick_style_behavior_on_init() {
    let behavior = CandlestickStyleBehavior {
        up_color: Color::rgb(0.11, 0.71, 0.43),
        down_color: Color::rgb(0.81, 0.18, 0.21),
        wick_color: None,
        wick_up_color: Color::rgb(0.06, 0.52, 0.30),
        wick_down_color: Color::rgb(0.58, 0.11, 0.15),
        border_color: None,
        border_up_color: Color::rgb(0.03, 0.40, 0.24),
        border_down_color: Color::rgb(0.43, 0.08, 0.11),
        body_mode: CandlestickBodyMode::HollowUp,
        wick_width_px: 2.0,
        border_width_px: 1.5,
        show_wicks: true,
        show_borders: true,
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_candlestick_style_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.candlestick_style_behavior(), behavior);
}

#[test]
fn chart_engine_config_json_without_candlestick_style_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(config.candlestick_style_behavior, None);
}

#[test]
fn chart_engine_config_json_parses_candlestick_style_behavior() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "candlestick_style_behavior": {
    "up_color": { "red": 0.11, "green": 0.71, "blue": 0.43, "alpha": 1.0 },
    "down_color": { "red": 0.81, "green": 0.18, "blue": 0.21, "alpha": 1.0 },
    "wick_color": { "red": 0.77, "green": 0.33, "blue": 0.19, "alpha": 1.0 },
    "wick_up_color": { "red": 0.06, "green": 0.52, "blue": 0.30, "alpha": 1.0 },
    "wick_down_color": { "red": 0.58, "green": 0.11, "blue": 0.15, "alpha": 1.0 },
    "border_color": { "red": 0.22, "green": 0.30, "blue": 0.65, "alpha": 1.0 },
    "border_up_color": { "red": 0.03, "green": 0.40, "blue": 0.24, "alpha": 1.0 },
    "border_down_color": { "red": 0.43, "green": 0.08, "blue": 0.11, "alpha": 1.0 },
    "body_mode": "HollowUp",
    "wick_width_px": 2.0,
    "border_width_px": 1.5,
    "show_wicks": true,
    "show_borders": false
  }
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.candlestick_style_behavior,
        Some(CandlestickStyleBehavior {
            up_color: Color::rgba(0.11, 0.71, 0.43, 1.0),
            down_color: Color::rgba(0.81, 0.18, 0.21, 1.0),
            wick_color: Some(Color::rgba(0.77, 0.33, 0.19, 1.0)),
            wick_up_color: Color::rgba(0.06, 0.52, 0.30, 1.0),
            wick_down_color: Color::rgba(0.58, 0.11, 0.15, 1.0),
            border_color: Some(Color::rgba(0.22, 0.30, 0.65, 1.0)),
            border_up_color: Color::rgba(0.03, 0.40, 0.24, 1.0),
            border_down_color: Color::rgba(0.43, 0.08, 0.11, 1.0),
            body_mode: CandlestickBodyMode::HollowUp,
            wick_width_px: 2.0,
            border_width_px: 1.5,
            show_wicks: true,
            show_borders: false,
        })
    );
}

#[test]
fn chart_engine_config_shared_wick_border_colors_override_directional_fields_on_init() {
    let behavior = CandlestickStyleBehavior {
        up_color: Color::rgb(0.11, 0.71, 0.43),
        down_color: Color::rgb(0.81, 0.18, 0.21),
        wick_color: Some(Color::rgb(0.77, 0.33, 0.19)),
        wick_up_color: Color::rgb(0.06, 0.52, 0.30),
        wick_down_color: Color::rgb(0.58, 0.11, 0.15),
        border_color: Some(Color::rgb(0.22, 0.30, 0.65)),
        border_up_color: Color::rgb(0.03, 0.40, 0.24),
        border_down_color: Color::rgb(0.43, 0.08, 0.11),
        body_mode: CandlestickBodyMode::Solid,
        wick_width_px: 2.0,
        border_width_px: 1.5,
        show_wicks: true,
        show_borders: true,
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_candlestick_style_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    let style = engine.render_style();
    assert_eq!(
        style.candlestick_wick_up_color,
        behavior.wick_color.expect("wick")
    );
    assert_eq!(
        style.candlestick_wick_down_color,
        behavior.wick_color.expect("wick")
    );
    assert_eq!(
        style.candlestick_border_up_color,
        behavior.border_color.expect("border")
    );
    assert_eq!(
        style.candlestick_border_down_color,
        behavior.border_color.expect("border")
    );
}
