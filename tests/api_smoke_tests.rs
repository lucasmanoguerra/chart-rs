use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::interaction::InteractionMode;
use chart_rs::render::NullRenderer;

#[test]
fn engine_smoke_flow() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(800, 600), 0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);
    engine.append_point(DataPoint::new(3.0, 30.0));

    engine.pointer_move(120.0, 40.0);
    engine.pan_start();
    assert_eq!(engine.interaction_mode(), InteractionMode::Panning);
    engine.pan_end();
    assert_eq!(engine.interaction_mode(), InteractionMode::Idle);

    engine.render().expect("render should succeed");
    assert_eq!(engine.points().len(), 3);
}
