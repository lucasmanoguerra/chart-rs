use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{OhlcBar, PriceScale, TimeScale, Viewport, project_bars};
use chart_rs::render::NullRenderer;

#[test]
fn bar_projection_returns_empty_for_empty_series() {
    let viewport = Viewport::new(1000, 500);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");

    let projected = project_bars(&[], time_scale, price_scale, viewport, 8.0).expect("project");
    assert!(projected.is_empty());
}

#[test]
fn bar_projection_rejects_invalid_tick_width() {
    let viewport = Viewport::new(1000, 500);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");
    let bars = vec![OhlcBar::new(5.0, 40.0, 60.0, 30.0, 50.0).expect("valid ohlc")];

    let err = project_bars(&bars, time_scale, price_scale, viewport, 0.0)
        .expect_err("must reject width <= 0");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn bar_projection_is_deterministic() {
    let viewport = Viewport::new(1000, 500);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");

    let bars = vec![OhlcBar::new(5.0, 40.0, 60.0, 30.0, 50.0).expect("valid ohlc")];
    let projected = project_bars(&bars, time_scale, price_scale, viewport, 12.0).expect("project");

    assert_eq!(projected.len(), 1);
    let b = projected[0];

    assert!((b.center_x - 500.0).abs() <= 1e-9);
    assert!((b.open_x - 494.0).abs() <= 1e-9);
    assert!((b.close_x - 506.0).abs() <= 1e-9);
    assert!((b.high_y - 200.0).abs() <= 1e-9);
    assert!((b.low_y - 350.0).abs() <= 1e-9);
    assert!((b.open_y - 300.0).abs() <= 1e-9);
    assert!((b.close_y - 250.0).abs() <= 1e-9);
}

#[test]
fn project_visible_bars_uses_visible_range() {
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
        .project_visible_bars(6.0)
        .expect("visible projection");
    assert_eq!(projected.len(), 2);

    // For visible range 25..60 and width 700:
    // x(30)=100, x(50)=500.
    assert!((projected[0].center_x - 100.0).abs() <= 1e-9);
    assert!((projected[1].center_x - 500.0).abs() <= 1e-9);
}

#[test]
fn project_visible_bars_with_overscan_includes_neighbors() {
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
        .project_visible_bars(6.0)
        .expect("visible projection");
    let overscan = engine
        .project_visible_bars_with_overscan(6.0, 0.2)
        .expect("overscan projection");

    assert_eq!(baseline.len(), 2);
    assert_eq!(overscan.len(), 4);
}

#[test]
fn project_visible_bars_with_overscan_rejects_invalid_ratio() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(700, 400), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .project_visible_bars_with_overscan(6.0, -0.5)
        .expect_err("invalid overscan must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
