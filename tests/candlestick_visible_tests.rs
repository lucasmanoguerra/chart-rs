use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn project_visible_candles_uses_visible_range() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(700, 400), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_candles(vec![
        OhlcBar::new(10.0, 20.0, 22.0, 18.0, 21.0).expect("c1"),
        OhlcBar::new(30.0, 30.0, 33.0, 28.0, 29.0).expect("c2"),
        OhlcBar::new(50.0, 40.0, 44.0, 39.0, 43.0).expect("c3"),
        OhlcBar::new(90.0, 70.0, 75.0, 69.0, 72.0).expect("c4"),
    ]);
    engine
        .set_time_visible_range(25.0, 60.0)
        .expect("set visible range");

    let projected = engine
        .project_visible_candles(6.0)
        .expect("visible projection");
    assert_eq!(projected.len(), 2);

    // For visible range 25..60 and width 700:
    // x(30)=100, x(50)=500.
    assert!((projected[0].center_x - 100.0).abs() <= 1e-9);
    assert!((projected[1].center_x - 500.0).abs() <= 1e-9);
}

#[test]
fn project_visible_candles_with_overscan_includes_neighbors() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(700, 400), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_candles(vec![
        OhlcBar::new(20.0, 20.0, 22.0, 18.0, 21.0).expect("c1"),
        OhlcBar::new(30.0, 30.0, 33.0, 28.0, 29.0).expect("c2"),
        OhlcBar::new(50.0, 40.0, 44.0, 39.0, 43.0).expect("c3"),
        OhlcBar::new(65.0, 70.0, 75.0, 69.0, 72.0).expect("c4"),
    ]);
    engine
        .set_time_visible_range(25.0, 60.0)
        .expect("set visible range");

    let baseline = engine
        .project_visible_candles(6.0)
        .expect("visible projection");
    let overscan = engine
        .project_visible_candles_with_overscan(6.0, 0.2)
        .expect("overscan projection");

    assert_eq!(baseline.len(), 2);
    assert_eq!(overscan.len(), 4);
}

#[test]
fn project_visible_candles_with_overscan_rejects_invalid_ratio() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(700, 400), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .project_visible_candles_with_overscan(6.0, -0.5)
        .expect_err("invalid overscan must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
