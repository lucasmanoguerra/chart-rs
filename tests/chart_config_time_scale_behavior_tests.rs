use chart_rs::api::{ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_time_scale_bootstrap_fields() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    assert_eq!(
        config.time_scale_navigation_behavior,
        TimeScaleNavigationBehavior::default()
    );
    assert_eq!(config.time_scale_right_offset_px, None);
}

#[test]
fn chart_engine_config_applies_right_offset_px_on_init() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_scale_right_offset_px(Some(100.0));
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");
    let (start, end) = engine.time_visible_range();
    assert!((start - 10.0).abs() <= 1e-9);
    assert!((end - 110.0).abs() <= 1e-9);
}

#[test]
fn chart_engine_config_rejects_invalid_right_offset_px() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_scale_right_offset_px(Some(-1.0));
    let renderer = NullRenderer::default();
    match ChartEngine::new(renderer, config) {
        Ok(_) => panic!("invalid right offset must fail"),
        Err(err) => assert!(matches!(err, chart_rs::ChartError::InvalidData(_))),
    }
}

#[test]
fn chart_engine_config_json_without_time_scale_fields_uses_defaults() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;
    let config = ChartEngineConfig::from_json_str(json).expect("parse config");
    assert_eq!(
        config.time_scale_navigation_behavior,
        TimeScaleNavigationBehavior::default()
    );
    assert_eq!(config.time_scale_right_offset_px, None);
}
