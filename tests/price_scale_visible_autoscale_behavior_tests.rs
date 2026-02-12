use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 10.0).with_price_domain(0.0, 1000.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn autoscale_price_from_visible_candles_uses_visible_window_envelope() {
    let mut engine = build_engine();
    engine.set_candles(vec![
        OhlcBar::new(0.0, 900.0, 1_000.0, 880.0, 960.0).expect("candle"),
        OhlcBar::new(1.0, 20.0, 24.0, 18.0, 22.0).expect("candle"),
        OhlcBar::new(2.0, 25.0, 28.0, 22.0, 24.0).expect("candle"),
    ]);

    engine
        .set_time_visible_range(0.5, 2.5)
        .expect("set visible range");
    engine
        .autoscale_price_from_visible_candles()
        .expect("autoscale visible candles");

    let (min, max) = engine.price_domain();
    assert!(max < 100.0);
    assert!(min > 10.0);
}

#[test]
fn autoscale_price_from_visible_data_uses_visible_points_only() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 950.0),
        DataPoint::new(1.0, 40.0),
        DataPoint::new(2.0, 48.0),
        DataPoint::new(3.0, 44.0),
    ]);

    engine
        .set_time_visible_range(0.9, 3.1)
        .expect("set visible range");
    engine
        .autoscale_price_from_visible_data()
        .expect("autoscale visible data");

    let (min, max) = engine.price_domain();
    assert!(max < 80.0);
    assert!(min > 30.0);
}

#[test]
fn autoscale_price_from_visible_window_is_noop_when_window_has_no_data() {
    let mut engine = build_engine();
    engine.set_data(vec![DataPoint::new(0.0, 10.0), DataPoint::new(1.0, 20.0)]);
    let before = engine.price_domain();

    engine
        .set_time_visible_range(100.0, 200.0)
        .expect("set empty visible window");
    engine
        .autoscale_price_from_visible_data()
        .expect("visible autoscale no-op");

    assert_eq!(engine.price_domain(), before);
}
