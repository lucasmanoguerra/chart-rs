use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleScrollZoomBehavior, TimeScaleZoomLimitBehavior,
};
use chart_rs::core::{DataPoint, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine(config: ChartEngineConfig) -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    ChartEngine::new(renderer, config).expect("engine init")
}

fn seed_points() -> Vec<DataPoint> {
    (0..300)
        .map(|index| DataPoint::new(index as f64, 100.0 + index as f64 * 0.1))
        .collect()
}

#[test]
fn chart_engine_config_defaults_time_scale_zoom_behaviors() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    assert_eq!(
        config.time_scale_scroll_zoom_behavior,
        TimeScaleScrollZoomBehavior::default()
    );
    assert_eq!(
        config.time_scale_zoom_limit_behavior,
        TimeScaleZoomLimitBehavior::default()
    );
}

#[test]
fn chart_engine_config_applies_scroll_zoom_right_bar_policy() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_scale_scroll_zoom_behavior(TimeScaleScrollZoomBehavior {
            right_bar_stays_on_scroll: true,
        });
    let mut engine = build_engine(config);
    engine.set_data(seed_points());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");

    let (_, right_before) = engine.time_visible_range();
    let _ = engine
        .wheel_zoom_time_visible(-120.0, 250.0, 0.2, 0.5)
        .expect("wheel zoom");
    let (_, right_after) = engine.time_visible_range();
    assert!((right_after - right_before).abs() <= 1e-9);
}

#[test]
fn chart_engine_config_applies_zoom_limit_policy() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 20.0,
            max_bar_spacing_px: None,
        });
    let mut engine = build_engine(config);
    engine.set_data(seed_points());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");

    engine
        .set_time_visible_range(-500.0, 500.0)
        .expect("set visible range");
    let (start, end) = engine.time_visible_range();
    assert!(((end - start) - 50.0).abs() <= 1e-9);
}
