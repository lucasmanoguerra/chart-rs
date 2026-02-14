use chart_rs::ChartError;
use chart_rs::api::{
    ChartEngine, ChartEngineConfig, InvalidationTopic, TimeScaleNavigationBehavior,
    TimeScaleRealtimeAppendBehavior,
};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::lwc::model::TimeScaleInvalidationType;
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("disable default spacing navigation");
    engine
}

fn seed_points() -> Vec<DataPoint> {
    (0..6)
        .map(|index| DataPoint::new(index as f64 * 10.0, 100.0 + index as f64))
        .collect()
}

#[test]
fn default_realtime_append_behavior_contract() {
    let engine = build_engine();
    let behavior = engine.time_scale_realtime_append_behavior();
    assert!(behavior.preserve_right_edge_on_append);
    assert!((behavior.right_edge_tolerance_bars - 0.75).abs() <= 1e-12);
}

#[test]
fn default_behavior_tracks_right_edge_when_at_tail() {
    let mut engine = build_engine();

    engine.append_point(DataPoint::new(101.0, 1.0));

    let (full_start, full_end) = engine.time_full_range();
    let (visible_start, visible_end) = engine.time_visible_range();

    assert!((full_start - 0.0).abs() <= 1e-9);
    assert!((full_end - 101.0).abs() <= 1e-9);
    assert!((visible_start - 1.0).abs() <= 1e-9);
    assert!((visible_end - 101.0).abs() <= 1e-9);
}

#[test]
fn append_does_not_shift_when_viewport_is_away_from_tail_beyond_tolerance() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .set_time_visible_range(0.0, 90.0)
        .expect("set visible range");

    engine.append_point(DataPoint::new(110.0, 1.0));

    let (_, full_end) = engine.time_full_range();
    let (visible_start, visible_end) = engine.time_visible_range();
    assert!((full_end - 110.0).abs() <= 1e-9);
    assert!((visible_start - 0.0).abs() <= 1e-9);
    assert!((visible_end - 90.0).abs() <= 1e-9);
}

#[test]
fn append_shifts_continuously_when_within_right_edge_tolerance() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .set_time_visible_range(0.0, 93.0)
        .expect("set visible range");

    engine.append_point(DataPoint::new(110.0, 1.0));

    let (visible_start, visible_end) = engine.time_visible_range();
    assert!((visible_start - 10.0).abs() <= 1e-9);
    assert!((visible_end - 103.0).abs() <= 1e-9);
}

#[test]
fn disable_preserve_right_edge_keeps_visible_range_fixed() {
    let mut engine = build_engine();
    engine
        .set_time_scale_realtime_append_behavior(TimeScaleRealtimeAppendBehavior {
            preserve_right_edge_on_append: false,
            right_edge_tolerance_bars: 0.75,
        })
        .expect("set realtime append behavior");

    engine.append_point(DataPoint::new(110.0, 1.0));

    let (visible_start, visible_end) = engine.time_visible_range();
    assert!((visible_start - 0.0).abs() <= 1e-9);
    assert!((visible_end - 100.0).abs() <= 1e-9);
}

#[test]
fn append_composes_with_right_offset_navigation() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .expect("set navigation behavior");

    let (_, end_before) = engine.time_visible_range();
    engine.append_point(DataPoint::new(110.0, 1.0));
    let (_, end_after) = engine.time_visible_range();

    assert!((end_before - 120.0).abs() <= 1e-9);
    assert!((end_after - 130.0).abs() <= 1e-9);
}

#[test]
fn append_composes_with_navigation_spacing_policy() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: Some(20.0),
        })
        .expect("set navigation behavior");

    engine.append_point(DataPoint::new(110.0, 1.0));

    let (_, full_end) = engine.time_full_range();
    let (start_after, end_after) = engine.time_visible_range();
    assert!((full_end - 110.0).abs() <= 1e-9);
    assert!((end_after - full_end).abs() <= 1e-9);
    assert!(((end_after - start_after) - 500.0).abs() <= 1e-9);
}

#[test]
fn append_candle_uses_same_realtime_policy() {
    let mut engine = build_engine();
    let candle = OhlcBar::new(110.0, 1.0, 2.0, 0.5, 1.5).expect("valid candle");

    engine.append_candle(candle);

    let (_, full_end) = engine.time_full_range();
    let (visible_start, visible_end) = engine.time_visible_range();
    assert!((full_end - 110.0).abs() <= 1e-9);
    assert!((visible_start - 10.0).abs() <= 1e-9);
    assert!((visible_end - 110.0).abs() <= 1e-9);
}

#[test]
fn invalid_realtime_tolerance_is_rejected() {
    let mut engine = build_engine();
    let err = engine
        .set_time_scale_realtime_append_behavior(TimeScaleRealtimeAppendBehavior {
            preserve_right_edge_on_append: true,
            right_edge_tolerance_bars: -0.5,
        })
        .expect_err("negative tolerance must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn realtime_append_with_right_edge_tracking_registers_apply_right_offset_invalidation() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine.clear_pending_invalidation();

    engine.append_point(DataPoint::new(110.0, 1.0));

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(
        kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset)
            || kinds.contains(&TimeScaleInvalidationType::ApplyRange)
    );
}

#[test]
fn realtime_append_without_right_edge_tracking_does_not_emit_time_scale_invalidation() {
    let mut engine = build_engine();
    engine
        .set_time_scale_realtime_append_behavior(TimeScaleRealtimeAppendBehavior {
            preserve_right_edge_on_append: false,
            right_edge_tolerance_bars: 0.75,
        })
        .expect("set realtime append behavior");
    engine.set_data(seed_points());
    engine.clear_pending_invalidation();

    engine.append_point(DataPoint::new(110.0, 1.0));

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.is_empty());
    assert!(!kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
    assert!(!engine.has_pending_invalidation_topic(InvalidationTopic::TimeScale));
}
