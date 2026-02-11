use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, PriceScaleMode, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn set_price_scale_mode_log_changes_mapping_shape() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(1.0, 1_000.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let linear_y1 = engine.map_price_to_pixel(1.0).expect("linear y1");
    let linear_y10 = engine.map_price_to_pixel(10.0).expect("linear y10");
    let linear_y100 = engine.map_price_to_pixel(100.0).expect("linear y100");
    let linear_d1 = linear_y1 - linear_y10;
    let linear_d2 = linear_y10 - linear_y100;
    assert!((linear_d1 - linear_d2).abs() > 1.0);

    engine
        .set_price_scale_mode(PriceScaleMode::Log)
        .expect("set log mode");
    assert_eq!(engine.price_scale_mode(), PriceScaleMode::Log);

    let log_y1 = engine.map_price_to_pixel(1.0).expect("log y1");
    let log_y10 = engine.map_price_to_pixel(10.0).expect("log y10");
    let log_y100 = engine.map_price_to_pixel(100.0).expect("log y100");
    let log_d1 = log_y1 - log_y10;
    let log_d2 = log_y10 - log_y100;
    assert!((log_d1 - log_d2).abs() <= 1e-6);
}

#[test]
fn set_price_scale_mode_log_rejects_non_positive_domain() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(-5.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    assert!(engine.set_price_scale_mode(PriceScaleMode::Log).is_err());
}

#[test]
fn autoscale_in_log_mode_preserves_mode_and_positive_domain() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(1.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_price_scale_mode(PriceScaleMode::Log)
        .expect("set log mode");

    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 5.0),
        DataPoint::new(2.0, 50.0),
    ]);
    engine
        .autoscale_price_from_data()
        .expect("autoscale in log mode");

    assert_eq!(engine.price_scale_mode(), PriceScaleMode::Log);
    let (min, max) = engine.price_domain();
    assert!(min > 0.0);
    assert!(max > min);

    let px = engine.map_price_to_pixel(5.0).expect("map price");
    let recovered = engine.map_pixel_to_price(px).expect("recover price");
    assert!((recovered - 5.0).abs() <= 1e-9);
}
