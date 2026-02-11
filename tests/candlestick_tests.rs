use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{OhlcBar, PriceScale, TimeScale, Viewport, project_candles};
use chart_rs::render::NullRenderer;

#[test]
fn invalid_ohlc_is_rejected() {
    let bar = OhlcBar::new(1.0, 120.0, 110.0, 90.0, 100.0);
    assert!(bar.is_err());
}

#[test]
fn candlestick_projection_is_deterministic() {
    let viewport = Viewport::new(1000, 500);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");

    let bars = vec![OhlcBar::new(5.0, 40.0, 60.0, 30.0, 50.0).expect("valid ohlc")];
    let projected =
        project_candles(&bars, time_scale, price_scale, viewport, 12.0).expect("projection");

    assert_eq!(projected.len(), 1);
    let c = projected[0];

    assert!((c.center_x - 500.0).abs() <= 1e-9);
    assert!((c.body_left - 494.0).abs() <= 1e-9);
    assert!((c.body_right - 506.0).abs() <= 1e-9);
    assert!((c.wick_top - 200.0).abs() <= 1e-9);
    assert!((c.wick_bottom - 350.0).abs() <= 1e-9);
    assert!((c.body_top - 250.0).abs() <= 1e-9);
    assert!((c.body_bottom - 300.0).abs() <= 1e-9);
    assert!(c.is_bullish);
}

#[test]
fn engine_autoscales_and_projects_candles() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 600), 0.0, 10.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_candles(vec![
        OhlcBar::new(1.0, 100.0, 110.0, 90.0, 105.0).expect("valid ohlc"),
        OhlcBar::new(2.0, 105.0, 120.0, 95.0, 98.0).expect("valid ohlc"),
    ]);

    engine
        .autoscale_price_from_candles()
        .expect("autoscale from candles");

    let (min, max) = engine.price_domain();
    assert_eq!(min, 90.0);
    assert_eq!(max, 120.0);

    let geometries = engine.project_candles(8.0).expect("project candles");
    assert_eq!(geometries.len(), 2);
    assert!(geometries[0].is_bullish);
    assert!(!geometries[1].is_bullish);
}
