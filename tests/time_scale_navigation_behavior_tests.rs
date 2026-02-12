use chart_rs::ChartError;
use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
};
use chart_rs::core::{DataPoint, OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

fn seed_points() -> Vec<DataPoint> {
    (0..10)
        .map(|index| DataPoint::new(index as f64 * 10.0, 100.0 + index as f64))
        .collect()
}

fn seed_candles() -> Vec<OhlcBar> {
    seed_points()
        .into_iter()
        .map(|point| OhlcBar::new(point.x, point.y, point.y + 1.0, point.y - 1.0, point.y))
        .collect::<Result<Vec<_>, _>>()
        .expect("valid candles")
}

fn prepare_fitted_engine() -> ChartEngine<NullRenderer> {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine.set_candles(seed_candles());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit to data");
    engine
}

#[test]
fn right_offset_bars_shifts_visible_window_keeping_current_span() {
    let mut engine = prepare_fitted_engine();

    let (start_before, end_before) = engine.time_visible_range();
    let span_before = end_before - start_before;
    let (_, full_end) = engine.time_full_range();

    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .expect("set navigation behavior");

    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - (full_end + 20.0)).abs() <= 1e-9);
    assert!((start_after - (end_after - span_before)).abs() <= 1e-9);
}

#[test]
fn bar_spacing_sets_deterministic_visible_span() {
    let mut engine = prepare_fitted_engine();
    let (_, full_end) = engine.time_full_range();

    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: Some(20.0),
        })
        .expect("set navigation behavior");

    let (start_after, end_after) = engine.time_visible_range();
    let span_after = end_after - start_after;

    assert!((end_after - full_end).abs() <= 1e-9);
    assert!((span_after - 500.0).abs() <= 1e-9);
}

#[test]
fn right_offset_and_spacing_compose() {
    let mut engine = prepare_fitted_engine();
    let (_, full_end) = engine.time_full_range();

    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 3.0,
            bar_spacing_px: Some(20.0),
        })
        .expect("set navigation behavior");

    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - (full_end + 30.0)).abs() <= 1e-9);
    assert!(((end_after - start_after) - 500.0).abs() <= 1e-9);
}

#[test]
fn fixed_right_edge_clamps_positive_right_offset() {
    let mut engine = prepare_fitted_engine();

    let (start_before, end_before) = engine.time_visible_range();
    let span_before = end_before - start_before;
    let (_, full_end) = engine.time_full_range();

    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        })
        .expect("set edge behavior");

    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .expect("set navigation behavior");

    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - full_end).abs() <= 1e-9);
    assert!((start_after - (end_after - span_before)).abs() <= 1e-9);
}

#[test]
fn invalid_bar_spacing_is_rejected() {
    let mut engine = prepare_fitted_engine();
    let err = engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: Some(0.0),
        })
        .expect_err("zero spacing must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn fit_time_to_data_applies_navigation_behavior_automatically() {
    let mut engine = build_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 1.0,
            bar_spacing_px: Some(25.0),
        })
        .expect("set navigation behavior");

    engine.set_data(seed_points());
    engine.set_candles(seed_candles());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit to data");

    let (_, full_end) = engine.time_full_range();
    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - (full_end + 10.0)).abs() <= 1e-9);
    assert!(((end_after - start_after) - 400.0).abs() <= 1e-9);
}

#[test]
fn point_only_series_can_drive_navigation_step_resolution() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit to data");

    let (_, full_end) = engine.time_full_range();

    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .expect("set navigation behavior");

    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - (full_end + 20.0)).abs() <= 1e-9);
}
