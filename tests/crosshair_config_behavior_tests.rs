use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_crosshair_mode_to_magnet() {
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");
    assert_eq!(engine.crosshair_mode(), CrosshairMode::Magnet);
}

#[test]
fn chart_engine_config_can_boot_in_normal_crosshair_mode() {
    let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0)
        .with_price_domain(0.0, 100.0)
        .with_crosshair_mode(CrosshairMode::Normal);
    let renderer = NullRenderer::default();
    let mut engine = ChartEngine::new(renderer, config).expect("engine");

    engine.set_data(vec![DataPoint::new(2.0, 20.0), DataPoint::new(8.0, 80.0)]);
    let pointer_x = engine.map_x_to_pixel(2.1).expect("map x");
    engine.pointer_move(pointer_x, 120.0);

    let crosshair = engine.crosshair_state();
    assert!(crosshair.snapped_x.is_none());
    assert!(crosshair.snapped_time.is_none());
}

#[test]
fn chart_engine_config_json_without_crosshair_mode_uses_default() {
    let json = r#"{
  "viewport": { "width": 800, "height": 500 },
  "time_start": 0.0,
  "time_end": 10.0,
  "price_min": 0.0,
  "price_max": 100.0
}"#;
    let config = ChartEngineConfig::from_json_str(json).expect("parse config");
    assert_eq!(config.crosshair_mode, CrosshairMode::Magnet);
}

#[test]
fn chart_engine_config_can_boot_in_hidden_crosshair_mode() {
    let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0)
        .with_price_domain(0.0, 100.0)
        .with_crosshair_mode(CrosshairMode::Hidden);
    let renderer = NullRenderer::default();
    let mut engine = ChartEngine::new(renderer, config).expect("engine");
    engine.set_data(vec![DataPoint::new(1.0, 10.0)]);
    engine.pointer_move(200.0, 120.0);
    assert!(!engine.crosshair_state().visible);
}
