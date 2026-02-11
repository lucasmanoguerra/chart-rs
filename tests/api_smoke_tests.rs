use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::interaction::InteractionMode;
use chart_rs::render::NullRenderer;

#[test]
fn engine_smoke_flow() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 600), 0.0, 100.0).with_price_domain(10.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);
    engine.append_point(DataPoint::new(3.0, 30.0));
    engine
        .autoscale_price_from_data()
        .expect("autoscale should succeed");

    engine.pointer_move(120.0, 40.0);
    engine.pan_start();
    assert_eq!(engine.interaction_mode(), InteractionMode::Panning);
    engine.pan_end();
    assert_eq!(engine.interaction_mode(), InteractionMode::Idle);

    engine.render().expect("render should succeed");
    assert_eq!(engine.points().len(), 3);

    let x_px = engine.map_x_to_pixel(3.0).expect("x to pixel");
    let x_back = engine.map_pixel_to_x(x_px).expect("pixel to x");
    assert!((x_back - 3.0).abs() <= 1e-9);

    let y_px = engine.map_price_to_pixel(30.0).expect("price to pixel");
    let y_back = engine.map_pixel_to_price(y_px).expect("pixel to price");
    assert!((y_back - 30.0).abs() <= 1e-9);

    let (min, max) = engine.price_domain();
    assert_eq!(min, 10.0);
    assert_eq!(max, 30.0);
}
