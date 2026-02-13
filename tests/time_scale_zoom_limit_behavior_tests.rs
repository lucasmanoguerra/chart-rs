use chart_rs::ChartError;
use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior, TimeScaleZoomLimitBehavior,
};
use chart_rs::core::{DataPoint, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

fn seed_points() -> Vec<DataPoint> {
    (0..300)
        .map(|index| DataPoint::new(index as f64, 100.0 + index as f64 * 0.1))
        .collect()
}

fn prepare_fitted_engine() -> ChartEngine<NullRenderer> {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
}

fn assert_bar_spacing_at_least(engine: &ChartEngine<NullRenderer>, min_spacing: f64) {
    let (start, end) = engine.time_visible_range();
    let span = (end - start).max(1e-9);
    let spacing = 1000.0 / span;
    assert!(spacing + 1e-9 >= min_spacing);
}

#[test]
fn default_zoom_limit_behavior_matches_contract() {
    let engine = build_engine();
    let behavior = engine.time_scale_zoom_limit_behavior();
    assert!((behavior.min_bar_spacing_px - 0.5).abs() <= 1e-12);
    assert_eq!(behavior.max_bar_spacing_px, None);
}

#[test]
fn invalid_zoom_limit_behavior_is_rejected() {
    let mut engine = prepare_fitted_engine();
    let err = engine
        .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 0.0,
            max_bar_spacing_px: None,
        })
        .expect_err("zero min spacing must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));

    let err = engine
        .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 5.0,
            max_bar_spacing_px: Some(4.0),
        })
        .expect_err("max spacing below min must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn min_spacing_clamps_zoom_out_span() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 20.0,
            max_bar_spacing_px: None,
        })
        .expect("set zoom limits");

    engine
        .set_time_visible_range(-500.0, 500.0)
        .expect("set visible range");
    let (start, end) = engine.time_visible_range();
    assert!(((end - start) - 50.0).abs() <= 1e-9);
    assert_bar_spacing_at_least(&engine, 20.0);
}

#[test]
fn max_spacing_clamps_zoom_in_span() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 0.5,
            max_bar_spacing_px: Some(40.0),
        })
        .expect("set zoom limits");

    engine
        .set_time_visible_range(49.0, 51.0)
        .expect("set visible range");
    let (start, end) = engine.time_visible_range();
    assert!(((end - start) - 25.0).abs() <= 1e-9);
}

#[test]
fn navigation_spacing_below_min_is_clamped_with_right_edge_preserved() {
    let mut engine = prepare_fitted_engine();
    let (_, full_end) = engine.time_full_range();

    engine
        .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 20.0,
            max_bar_spacing_px: None,
        })
        .expect("set zoom limits");
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: Some(5.0),
        })
        .expect("set navigation behavior");

    let (start, end) = engine.time_visible_range();
    assert!(((end - start) - 50.0).abs() <= 1e-9);
    assert!((end - full_end).abs() <= 1e-9);
}

#[test]
fn zoom_limit_clamp_preserves_right_offset_pixels_priority() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_right_offset_px(Some(120.0))
        .expect("set right offset px");

    engine
        .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 20.0,
            max_bar_spacing_px: None,
        })
        .expect("set zoom limits");

    let (_, full_end) = engine.time_full_range();
    let (start, end) = engine.time_visible_range();
    let span = end - start;
    let expected_offset = span * (120.0 / 1000.0);
    assert!((span - 50.0).abs() <= 1e-9);
    assert!(((end - full_end) - expected_offset).abs() <= 1e-9);
}
