use chart_rs::api::{ChartEngine, ChartEngineConfig, LastPriceBehavior, LastPriceSourceMode};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_last_price_behavior_to_none() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(config.last_price_behavior, None);
}

#[test]
fn chart_engine_config_applies_last_price_behavior_on_init() {
    let behavior = LastPriceBehavior {
        show_line: false,
        show_label: true,
        use_trend_color: true,
        source_mode: LastPriceSourceMode::LatestVisible,
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_last_price_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.last_price_behavior(), behavior);
}

#[test]
fn last_price_behavior_takes_precedence_over_last_price_source_mode() {
    let behavior = LastPriceBehavior {
        show_line: true,
        show_label: true,
        use_trend_color: false,
        source_mode: LastPriceSourceMode::LatestData,
    };
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_last_price_source_mode(LastPriceSourceMode::LatestVisible)
        .with_last_price_behavior(behavior);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.last_price_behavior(), behavior);
}

#[test]
fn chart_engine_config_json_without_last_price_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(config.last_price_behavior, None);
}

#[test]
fn chart_engine_config_json_parses_last_price_behavior() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "last_price_behavior": {
    "show_line": false,
    "show_label": false,
    "use_trend_color": true,
    "source_mode": "LatestVisible"
  }
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.last_price_behavior,
        Some(LastPriceBehavior {
            show_line: false,
            show_label: false,
            use_trend_color: true,
            source_mode: LastPriceSourceMode::LatestVisible,
        })
    );
}
