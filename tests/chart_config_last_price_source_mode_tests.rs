use chart_rs::api::{ChartEngine, ChartEngineConfig, LastPriceSourceMode};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_last_price_source_mode() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(
        config.last_price_source_mode,
        LastPriceSourceMode::LatestData
    );
}

#[test]
fn chart_engine_config_applies_last_price_source_mode_on_init() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_last_price_source_mode(LastPriceSourceMode::LatestVisible);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(
        engine.render_style().last_price_source_mode,
        LastPriceSourceMode::LatestVisible
    );
}

#[test]
fn chart_engine_config_json_without_last_price_source_mode_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.last_price_source_mode,
        LastPriceSourceMode::LatestData
    );
}

#[test]
fn chart_engine_config_json_parses_last_price_source_mode() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0,
  "last_price_source_mode": "LatestVisible"
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.last_price_source_mode,
        LastPriceSourceMode::LatestVisible
    );
}
