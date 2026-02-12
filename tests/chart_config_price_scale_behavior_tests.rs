use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{PriceScaleMode, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_price_scale_bootstrap_fields() {
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    assert_eq!(config.price_scale_mode, PriceScaleMode::Linear);
    assert!(!config.price_scale_inverted);
    assert!((config.price_scale_margins.top_margin_ratio - 0.2).abs() <= 1e-12);
    assert!((config.price_scale_margins.bottom_margin_ratio - 0.1).abs() <= 1e-12);
}

#[test]
fn chart_engine_config_applies_price_scale_bootstrap_fields() {
    let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0)
        .with_price_domain(1.0, 100.0)
        .with_price_scale_mode(PriceScaleMode::Log)
        .with_price_scale_inverted(true)
        .with_price_scale_margins(0.1, 0.2);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(engine.price_scale_mode(), PriceScaleMode::Log);
    assert!(engine.price_scale_inverted());
    let margins = engine.price_scale_margin_behavior();
    assert!((margins.top_margin_ratio - 0.1).abs() <= 1e-12);
    assert!((margins.bottom_margin_ratio - 0.2).abs() <= 1e-12);

    let px = engine.map_price_to_pixel(10.0).expect("map");
    let roundtrip = engine.map_pixel_to_price(px).expect("roundtrip");
    assert!((roundtrip - 10.0).abs() <= 1e-9);
}

#[test]
fn chart_engine_config_json_without_price_scale_fields_uses_defaults() {
    let json = r#"{
  "viewport": { "width": 800, "height": 500 },
  "time_start": 0.0,
  "time_end": 10.0,
  "price_min": 1.0,
  "price_max": 100.0,
  "crosshair_mode": "Magnet"
}"#;
    let config = ChartEngineConfig::from_json_str(json).expect("parse config");
    assert_eq!(config.price_scale_mode, PriceScaleMode::Linear);
    assert!(!config.price_scale_inverted);
    assert!((config.price_scale_margins.top_margin_ratio - 0.2).abs() <= 1e-12);
    assert!((config.price_scale_margins.bottom_margin_ratio - 0.1).abs() <= 1e-12);
}

#[test]
fn chart_engine_new_rejects_invalid_bootstrap_price_scale_margins() {
    let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0)
        .with_price_domain(1.0, 100.0)
        .with_price_scale_margins(0.6, 0.4);
    let renderer = NullRenderer::default();
    match ChartEngine::new(renderer, config) {
        Ok(_) => panic!("invalid margins should fail"),
        Err(err) => assert!(matches!(err, chart_rs::ChartError::InvalidData(_))),
    }
}
