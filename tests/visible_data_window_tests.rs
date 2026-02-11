use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn visible_points_and_candles_use_current_visible_range() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1200, 600), 0.0, 100.0).with_price_domain(0.0, 200.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![
        DataPoint::new(5.0, 10.0),
        DataPoint::new(20.0, 20.0),
        DataPoint::new(50.0, 50.0),
        DataPoint::new(90.0, 90.0),
    ]);
    engine.set_candles(vec![
        OhlcBar::new(10.0, 9.0, 12.0, 8.0, 11.0).expect("valid candle"),
        OhlcBar::new(25.0, 20.0, 26.0, 19.0, 24.0).expect("valid candle"),
        OhlcBar::new(80.0, 70.0, 82.0, 68.0, 79.0).expect("valid candle"),
    ]);

    engine
        .set_time_visible_range(15.0, 60.0)
        .expect("set visible range");

    let visible_points = engine.visible_points();
    let visible_candles = engine.visible_candles();

    assert_eq!(visible_points.len(), 2);
    assert!((visible_points[0].x - 20.0).abs() <= 1e-9);
    assert!((visible_points[1].x - 50.0).abs() <= 1e-9);

    assert_eq!(visible_candles.len(), 1);
    assert!((visible_candles[0].time - 25.0).abs() <= 1e-9);
}

#[test]
fn visible_window_overscan_includes_neighbors() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1200, 600), 0.0, 100.0).with_price_domain(0.0, 200.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![
        DataPoint::new(8.0, 8.0),
        DataPoint::new(12.0, 12.0),
        DataPoint::new(88.0, 88.0),
        DataPoint::new(92.0, 92.0),
    ]);
    engine
        .set_time_visible_range(10.0, 90.0)
        .expect("set visible range");

    let baseline = engine.visible_points();
    let overscanned = engine
        .visible_points_with_overscan(0.05)
        .expect("overscan points");

    assert_eq!(baseline.len(), 2);
    assert_eq!(overscanned.len(), 4);
}

#[test]
fn visible_window_overscan_rejects_invalid_ratio() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1200, 600), 0.0, 100.0).with_price_domain(0.0, 200.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .visible_points_with_overscan(-0.1)
        .expect_err("negative overscan must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
