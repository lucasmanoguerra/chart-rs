use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, PriceScaleMode, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine_with_domain(price_min: f64, price_max: f64) -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(price_min, price_max);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn price_scale_is_not_inverted_by_default() {
    let engine = build_engine_with_domain(0.0, 100.0);
    assert!(!engine.price_scale_inverted());
}

#[test]
fn enabling_invert_scale_flips_pixel_direction() {
    let mut engine = build_engine_with_domain(0.0, 100.0);
    let low_normal = engine.map_price_to_pixel(0.0).expect("map low");
    let high_normal = engine.map_price_to_pixel(100.0).expect("map high");
    assert!(high_normal < low_normal);

    engine.set_price_scale_inverted(true);
    let low_inverted = engine.map_price_to_pixel(0.0).expect("map low inverted");
    let high_inverted = engine.map_price_to_pixel(100.0).expect("map high inverted");
    assert!(high_inverted > low_inverted);
}

#[test]
fn invert_scale_preserves_roundtrip_mapping() {
    let mut engine = build_engine_with_domain(0.0, 100.0);
    engine.set_price_scale_inverted(true);

    let values = [0.0, 10.0, 55.0, 100.0];
    for value in values {
        let px = engine.map_price_to_pixel(value).expect("map price");
        let back = engine.map_pixel_to_price(px).expect("map pixel");
        assert!((back - value).abs() <= 1e-9);
    }
}

#[test]
fn invert_scale_is_preserved_across_mode_switch_and_autoscale() {
    let mut engine = build_engine_with_domain(1.0, 200.0);
    engine.set_price_scale_inverted(true);
    assert!(engine.price_scale_inverted());

    engine
        .set_price_scale_mode(PriceScaleMode::Log)
        .expect("switch to log");
    assert!(engine.price_scale_inverted());

    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(1.0, 50.0),
        DataPoint::new(2.0, 120.0),
    ]);
    engine
        .autoscale_price_from_data()
        .expect("autoscale from points");
    assert!(engine.price_scale_inverted());
}

#[test]
fn invert_scale_is_preserved_across_candle_autoscale() {
    let mut engine = build_engine_with_domain(1.0, 200.0);
    engine.set_price_scale_inverted(true);

    engine.set_candles(vec![
        OhlcBar::new(0.0, 10.0, 20.0, 9.0, 18.0).expect("bar"),
        OhlcBar::new(1.0, 18.0, 28.0, 17.0, 24.0).expect("bar"),
    ]);
    engine
        .autoscale_price_from_candles()
        .expect("autoscale candles");
    assert!(engine.price_scale_inverted());
}
