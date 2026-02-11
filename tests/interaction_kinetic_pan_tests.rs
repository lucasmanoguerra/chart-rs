use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::Viewport;
use chart_rs::interaction::KineticPanConfig;
use chart_rs::render::NullRenderer;

#[test]
fn wheel_pan_translates_visible_range_and_preserves_span() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let before = engine.time_visible_range();
    let delta = engine
        .wheel_pan_time_visible(120.0, 0.1)
        .expect("wheel pan");
    assert!((delta - 10.0).abs() <= 1e-9);
    let after = engine.time_visible_range();
    assert!((after.0 - 10.0).abs() <= 1e-9);
    assert!((after.1 - 110.0).abs() <= 1e-9);
    assert!(((after.1 - after.0) - (before.1 - before.0)).abs() <= 1e-9);

    let back = engine
        .wheel_pan_time_visible(-120.0, 0.1)
        .expect("wheel pan back");
    assert!((back + 10.0).abs() <= 1e-9);
    assert_eq!(engine.time_visible_range(), before);
}

#[test]
fn wheel_pan_zero_delta_is_noop() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let before = engine.time_visible_range();
    let delta = engine.wheel_pan_time_visible(0.0, 0.2).expect("no-op");
    assert!((delta - 0.0).abs() <= 1e-12);
    assert_eq!(engine.time_visible_range(), before);
}

#[test]
fn wheel_pan_rejects_invalid_inputs() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .wheel_pan_time_visible(f64::NAN, 0.2)
        .expect_err("nan delta must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));

    let err = engine
        .wheel_pan_time_visible(120.0, 0.0)
        .expect_err("invalid ratio must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn kinetic_pan_step_moves_range_and_decays_velocity() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_kinetic_pan_config(KineticPanConfig {
            decay_per_second: 0.5,
            stop_velocity_abs: 0.01,
        })
        .expect("set config");

    engine.start_kinetic_pan(20.0).expect("start kinetic");
    assert!(engine.kinetic_pan_state().active);

    let moved = engine.step_kinetic_pan(1.0).expect("step");
    assert!(moved);
    let range = engine.time_visible_range();
    assert!((range.0 - 20.0).abs() <= 1e-9);
    assert!((range.1 - 120.0).abs() <= 1e-9);
    assert!((engine.kinetic_pan_state().velocity_time_per_sec - 10.0).abs() <= 1e-9);
}

#[test]
fn kinetic_pan_stops_when_velocity_drops_below_threshold() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_kinetic_pan_config(KineticPanConfig {
            decay_per_second: 0.1,
            stop_velocity_abs: 5.0,
        })
        .expect("set config");

    engine.start_kinetic_pan(20.0).expect("start kinetic");
    let moved = engine.step_kinetic_pan(1.0).expect("step");
    assert!(moved);
    assert!(!engine.kinetic_pan_state().active);

    let before = engine.time_visible_range();
    let moved = engine.step_kinetic_pan(1.0).expect("step inactive");
    assert!(!moved);
    assert_eq!(engine.time_visible_range(), before);
}

#[test]
fn kinetic_pan_rejects_invalid_inputs() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .set_kinetic_pan_config(KineticPanConfig {
            decay_per_second: 1.0,
            stop_velocity_abs: 0.1,
        })
        .expect_err("decay must be < 1");
    assert!(matches!(err, ChartError::InvalidData(_)));

    let err = engine
        .start_kinetic_pan(f64::NAN)
        .expect_err("velocity must be finite");
    assert!(matches!(err, ChartError::InvalidData(_)));

    let err = engine
        .step_kinetic_pan(0.0)
        .expect_err("delta seconds must be > 0");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
