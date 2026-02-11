use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn crosshair_snaps_to_nearest_data_point() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![DataPoint::new(2.0, 20.0), DataPoint::new(8.0, 80.0)]);

    let near_x = engine.map_x_to_pixel(2.1).expect("x map");
    engine.pointer_move(near_x, 200.0);

    let crosshair = engine.crosshair_state();
    assert!(crosshair.visible);

    let snapped_x = crosshair.snapped_x.expect("snapped x");
    let snapped_y = crosshair.snapped_y.expect("snapped y");
    let snapped_time = crosshair.snapped_time.expect("snapped time");
    let snapped_price = crosshair.snapped_price.expect("snapped price");

    let expected_x = engine.map_x_to_pixel(2.0).expect("expected x");
    let expected_y = engine.map_price_to_pixel(20.0).expect("expected y");

    assert!((snapped_x - expected_x).abs() <= 1e-9);
    assert!((snapped_y - expected_y).abs() <= 1e-9);
    assert!((snapped_time - 2.0).abs() <= 1e-9);
    assert!((snapped_price - 20.0).abs() <= 1e-9);
}

#[test]
fn crosshair_snaps_to_nearest_candle_close() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_candles(vec![
        OhlcBar::new(3.0, 10.0, 20.0, 5.0, 15.0).expect("valid bar"),
        OhlcBar::new(7.0, 70.0, 80.0, 60.0, 75.0).expect("valid bar"),
    ]);

    let near_x = engine.map_x_to_pixel(7.05).expect("x map");
    engine.pointer_move(near_x, 220.0);

    let crosshair = engine.crosshair_state();
    let snapped_x = crosshair.snapped_x.expect("snapped x");
    let snapped_y = crosshair.snapped_y.expect("snapped y");
    let snapped_time = crosshair.snapped_time.expect("snapped time");
    let snapped_price = crosshair.snapped_price.expect("snapped price");

    let expected_x = engine.map_x_to_pixel(7.0).expect("expected x");
    let expected_y = engine.map_price_to_pixel(75.0).expect("expected y");

    assert!((snapped_x - expected_x).abs() <= 1e-9);
    assert!((snapped_y - expected_y).abs() <= 1e-9);
    assert!((snapped_time - 7.0).abs() <= 1e-9);
    assert!((snapped_price - 75.0).abs() <= 1e-9);
}

#[test]
fn pointer_leave_hides_crosshair() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![DataPoint::new(2.0, 20.0)]);
    engine.pointer_move(100.0, 200.0);
    assert!(engine.crosshair_state().visible);

    engine.pointer_leave();
    let crosshair = engine.crosshair_state();
    assert!(!crosshair.visible);
    assert!(crosshair.snapped_x.is_none());
    assert!(crosshair.snapped_y.is_none());
    assert!(crosshair.snapped_time.is_none());
    assert!(crosshair.snapped_price.is_none());
}
