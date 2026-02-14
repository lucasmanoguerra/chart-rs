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

fn assert_right_offset_px_relation(engine: &ChartEngine<NullRenderer>, right_offset_px: f64) {
    let (start, end) = engine.time_visible_range();
    let (_, full_end) = engine.time_full_range();
    let width = f64::from(engine.viewport().width).max(1.0);
    let span = end - start;
    let expected_offset = span * (right_offset_px / width);
    let observed_offset = end - full_end;
    let tolerance = span.abs() * 1e-6 + 1e-6;
    assert!((observed_offset - expected_offset).abs() <= tolerance);
}

#[test]
fn default_right_offset_px_is_none() {
    let engine = prepare_fitted_engine();
    assert_eq!(engine.time_scale_right_offset_px(), None);
}

#[test]
fn right_offset_px_overrides_right_offset_bars_without_spacing() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .expect("set navigation behavior");
    let (start_before, end_before) = engine.time_visible_range();
    let visible_span_before = end_before - start_before;
    engine
        .set_time_scale_right_offset_px(Some(100.0))
        .expect("set right offset px");

    let (_, full_end) = engine.time_full_range();
    let (_, end_after) = engine.time_visible_range();
    let expected_offset = (visible_span_before / 1000.0) * 100.0;
    assert!((end_after - (full_end + expected_offset)).abs() <= 1e-9);
}

#[test]
fn right_offset_px_with_spacing_uses_pixel_margin() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: Some(20.0),
        })
        .expect("set navigation behavior");
    engine
        .set_time_scale_right_offset_px(Some(100.0))
        .expect("set right offset px");

    let (_, full_end) = engine.time_full_range();
    let (start_after, end_after) = engine.time_visible_range();
    assert!((end_after - (full_end + 50.0)).abs() <= 1e-9);
    assert!(((end_after - start_after) - 500.0).abs() <= 1e-9);
}

#[test]
fn fixed_right_edge_clamps_positive_right_offset_px() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        })
        .expect("set edge behavior");
    engine
        .set_time_scale_right_offset_px(Some(120.0))
        .expect("set right offset px");

    let (_, full_end) = engine.time_full_range();
    let (_, end_after) = engine.time_visible_range();
    assert!((end_after - full_end).abs() <= 1e-9);
}

#[test]
fn invalid_right_offset_px_is_rejected() {
    let mut engine = prepare_fitted_engine();

    let err = engine
        .set_time_scale_right_offset_px(Some(-1.0))
        .expect_err("negative value must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));

    let err = engine
        .set_time_scale_right_offset_px(Some(f64::NAN))
        .expect_err("nan must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn zoom_around_pixel_preserves_right_offset_px_relation() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 1.0,
            bar_spacing_px: Some(6.0),
        })
        .expect("set navigation behavior");
    engine
        .set_time_scale_right_offset_px(Some(0.0))
        .expect("set right offset px");

    assert_right_offset_px_relation(&engine, 0.0);
    engine
        .zoom_time_visible_around_pixel(0.2, 0.0, 1e-6)
        .expect("zoom around pixel");
    assert_right_offset_px_relation(&engine, 0.0);
}

#[test]
fn zoom_around_time_preserves_right_offset_px_relation() {
    let mut engine = prepare_fitted_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 1.0,
            bar_spacing_px: Some(6.0),
        })
        .expect("set navigation behavior");
    engine
        .set_time_scale_right_offset_px(Some(0.0))
        .expect("set right offset px");

    assert_right_offset_px_relation(&engine, 0.0);
    engine
        .zoom_time_visible_around_time(0.2, 0.0, 1e-6)
        .expect("zoom around time");
    assert_right_offset_px_relation(&engine, 0.0);
}
