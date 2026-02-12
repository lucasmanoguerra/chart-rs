use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

fn engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn set_data_canonicalizes_order_and_duplicate_times() {
    let mut engine = engine();
    engine.set_data(vec![
        DataPoint::new(3.0, 30.0),
        DataPoint::new(1.0, 10.0),
        DataPoint::new(2.0, 20.0),
        DataPoint::new(2.0, 25.0),
        DataPoint::new(1.0, 15.0),
    ]);

    let points = engine.points();
    assert_eq!(points.len(), 3);
    assert_eq!(points[0], DataPoint::new(1.0, 15.0));
    assert_eq!(points[1], DataPoint::new(2.0, 25.0));
    assert_eq!(points[2], DataPoint::new(3.0, 30.0));
}

#[test]
fn set_data_filters_non_finite_samples() {
    let mut engine = engine();
    engine.set_data(vec![
        DataPoint::new(f64::NAN, 1.0),
        DataPoint::new(1.0, f64::INFINITY),
        DataPoint::new(2.0, 20.0),
    ]);

    let points = engine.points();
    assert_eq!(points.len(), 1);
    assert_eq!(points[0], DataPoint::new(2.0, 20.0));
}

#[test]
fn set_candles_canonicalizes_order_and_duplicate_times() {
    let mut engine = engine();

    let c1 = OhlcBar::new(1.0, 10.0, 15.0, 9.0, 12.0).expect("c1");
    let c2 = OhlcBar::new(2.0, 12.0, 16.0, 11.0, 13.0).expect("c2");
    let c2_replace = OhlcBar::new(2.0, 13.0, 18.0, 12.0, 17.0).expect("c2 replace");
    let c3 = OhlcBar::new(3.0, 17.0, 20.0, 16.0, 18.0).expect("c3");

    engine.set_candles(vec![c3, c1, c2, c2_replace]);

    let candles = engine.candles();
    assert_eq!(candles.len(), 3);
    assert_eq!(candles[0], c1);
    assert_eq!(candles[1], c2_replace);
    assert_eq!(candles[2], c3);
}

#[test]
fn set_candles_filters_invalid_samples() {
    let mut engine = engine();

    let valid = OhlcBar::new(3.0, 10.0, 12.0, 9.0, 11.0).expect("valid candle");
    let invalid_non_finite = OhlcBar {
        time: 1.0,
        open: f64::NAN,
        high: 11.0,
        low: 9.0,
        close: 10.0,
    };
    let invalid_range = OhlcBar {
        time: 2.0,
        open: 10.0,
        high: 9.0,
        low: 11.0,
        close: 10.0,
    };

    engine.set_candles(vec![invalid_non_finite, invalid_range, valid]);

    let candles = engine.candles();
    assert_eq!(candles.len(), 1);
    assert_eq!(candles[0], valid);
}
