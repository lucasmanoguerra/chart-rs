use chart_rs::api::{ChartEngine, ChartEngineConfig, PriceScaleRealtimeBehavior};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_price_scale_realtime_behavior() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let behavior = config.price_scale_realtime_behavior;
    assert!(behavior.autoscale_on_data_set);
    assert!(behavior.autoscale_on_data_update);
}

#[test]
fn chart_engine_config_applies_price_scale_realtime_behavior() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
            autoscale_on_data_set: true,
            autoscale_on_data_update: true,
        });
    let renderer = NullRenderer::default();
    let mut engine = ChartEngine::new(renderer, config).expect("engine");

    let before = engine.price_domain();
    engine.set_data(vec![DataPoint::new(0.0, 10.0), DataPoint::new(1.0, 20.0)]);
    let after_set = engine.price_domain();
    assert!((after_set.0 - before.0).abs() > 1e-9 || (after_set.1 - before.1).abs() > 1e-9);

    engine.append_point(DataPoint::new(2.0, 200.0));
    let after_update = engine.price_domain();
    assert!(after_update.1 > after_set.1);
}

#[test]
fn chart_engine_config_json_without_price_scale_realtime_behavior_uses_default() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;
    let config = ChartEngineConfig::from_json_str(json).expect("parse config");
    assert_eq!(
        config.price_scale_realtime_behavior,
        PriceScaleRealtimeBehavior::default()
    );
}
