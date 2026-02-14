use chart_rs::api::{
    ChartEngine, ChartEngineConfig, InvalidationLevel, InvalidationTopic, RenderStyle,
    TimeScaleNavigationBehavior, TimeScaleResizeAnchor, TimeScaleResizeBehavior,
    TimeScaleZoomLimitBehavior,
};
use chart_rs::core::{DataPoint, TimeScaleTuning, Viewport};
use chart_rs::lwc::model::TimeScaleInvalidationType;
use chart_rs::render::{Color, NullRenderer};

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0).with_price_domain(0.0, 10.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn engine_starts_with_full_invalidation_and_render_clears_it() {
    let mut engine = build_engine();

    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::Full);
    assert!(engine.has_pending_invalidation());

    let rendered = engine
        .render_if_invalidated()
        .expect("render if invalidated");
    assert!(rendered);
    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::None);

    let rendered_again = engine
        .render_if_invalidated()
        .expect("render if invalidated");
    assert!(!rendered_again);
}

#[test]
fn pointer_move_sets_cursor_and_data_update_escalates_to_full() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();

    engine.pointer_move(120.0, 80.0);
    assert_eq!(
        engine.pending_invalidation_level(),
        InvalidationLevel::Cursor
    );
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::Cursor));

    engine.append_point(DataPoint::new(1.0, 2.0));
    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::Full);
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::Series));
    assert!(!engine.has_pending_invalidation_topic(InvalidationTopic::TimeScale));
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::PriceScale));
}

#[test]
fn visible_range_change_marks_light_invalidation() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
    ]);
    engine.clear_pending_invalidation();

    engine
        .pan_time_visible_by_pixels(32.0)
        .expect("pan by pixels should work");
    assert_eq!(
        engine.pending_invalidation_level(),
        InvalidationLevel::Light
    );
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::TimeScale));
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::Axis));
    let snapshot = engine
        .lwc_pending_invalidation_snapshot()
        .expect("lwc snapshot");
    assert_eq!(snapshot.level, InvalidationLevel::Light);
    assert!(snapshot.time_scale_invalidation_count > 0);
}

#[test]
fn take_pending_invalidation_clears_queue() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();
    engine.pointer_move(10.0, 5.0);

    let pending = engine.take_pending_invalidation();
    assert_eq!(pending.level(), InvalidationLevel::Cursor);
    assert!(pending.has_topic(InvalidationTopic::Cursor));
    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::None);
    assert!(engine.lwc_pending_invalidation().is_none());
}

#[test]
fn pane_content_invalidation_carries_single_pane_target() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();
    let pane_id = engine.create_pane(1.0).expect("create pane");
    engine.clear_pending_invalidation();

    engine.set_points_pane(pane_id).expect("set points pane");

    let pending = engine.pending_invalidation();
    assert_eq!(pending.level(), InvalidationLevel::Light);
    assert_eq!(engine.pending_invalidation_pane_targets(), vec![pane_id]);
    assert!(pending.has_topic(InvalidationTopic::Series));
    assert!(pending.has_topic(InvalidationTopic::PaneLayout));
}

#[test]
fn pending_invalidation_pane_targets_reports_all_explicit_lwc_panes() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();

    let pane_a = engine.create_pane(1.0).expect("create pane a");
    let pane_b = engine.create_pane(1.0).expect("create pane b");
    engine.clear_pending_invalidation();

    engine.set_points_pane(pane_a).expect("set points pane");
    engine.set_candles_pane(pane_b).expect("set candles pane");

    let mut targets = engine.pending_invalidation_pane_targets();
    targets.sort_by_key(|pane_id| pane_id.raw());
    assert_eq!(targets, vec![pane_a, pane_b]);
}

#[test]
fn clear_pending_invalidation_clears_both_api_and_lwc_queues() {
    let mut engine = build_engine();

    assert!(engine.has_pending_invalidation());
    assert!(engine.lwc_pending_invalidation().is_some());

    engine.clear_pending_invalidation();

    assert!(!engine.has_pending_invalidation());
    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::None);
    assert!(engine.lwc_pending_invalidation().is_none());
}

#[test]
fn set_render_style_noop_when_style_is_identical() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();

    let same_style = engine.render_style();
    engine
        .set_render_style(same_style)
        .expect("setting identical style should succeed");

    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::None);
    assert!(!engine.has_pending_invalidation());
}

#[test]
fn set_render_style_non_layout_change_triggers_light_invalidation() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();

    let changed_style = RenderStyle {
        grid_line_color: Color::rgb(0.13, 0.2, 0.27),
        ..engine.render_style()
    };
    engine
        .set_render_style(changed_style)
        .expect("setting changed style should succeed");

    assert_eq!(
        engine.pending_invalidation_level(),
        InvalidationLevel::Light
    );
    assert!(engine.has_pending_invalidation());
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::Style));
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::Axis));
    assert!(engine.has_pending_invalidation_topic(InvalidationTopic::Series));
}

#[test]
fn set_render_style_layout_change_triggers_full_invalidation() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();

    let style = engine.render_style();
    let changed_style = RenderStyle {
        price_axis_width_px: style.price_axis_width_px + 10.0,
        ..style
    };
    engine
        .set_render_style(changed_style)
        .expect("setting layout-changing style should succeed");

    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::Full);
    assert!(engine.has_pending_invalidation());
}

#[test]
fn lwc_pending_invalidation_snapshot_exposes_level_and_pane_mapping() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();

    let pane_id = engine.create_pane(1.0).expect("create pane");
    engine.clear_pending_invalidation();
    engine.set_points_pane(pane_id).expect("set points pane");

    let snapshot = engine
        .lwc_pending_invalidation_snapshot()
        .expect("lwc snapshot");
    assert_eq!(snapshot.level, InvalidationLevel::Light);
    assert!(!snapshot.pane_invalidations.is_empty());
    assert!(
        snapshot
            .pane_invalidations
            .iter()
            .any(|pane| pane.pane_id == pane_id && pane.level == InvalidationLevel::Light)
    );
}

#[test]
fn time_scale_pan_registers_specific_lwc_time_scale_invalidation_kind() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
    ]);
    engine.clear_pending_invalidation();

    engine
        .pan_time_visible_by_pixels(24.0)
        .expect("pan by pixels should work");

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(!kinds.is_empty());
    assert!(
        kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset)
            || kinds.contains(&TimeScaleInvalidationType::ApplyRange)
    );
}

#[test]
fn fit_time_to_data_registers_lwc_fit_content_invalidation_kind() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
    ]);
    engine.clear_pending_invalidation();

    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time to data should work");

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::FitContent));
}

#[test]
fn reset_visible_range_registers_lwc_reset_invalidation_kind() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
    ]);
    engine
        .pan_time_visible_by_pixels(32.0)
        .expect("pan by pixels should work");
    engine.clear_pending_invalidation();

    engine.reset_time_visible_range();

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::Reset));
}

#[test]
fn zoom_registers_lwc_bar_spacing_and_right_offset_invalidations() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine.clear_pending_invalidation();

    engine
        .zoom_time_visible_around_pixel(1.2, 300.0, 1e-9)
        .expect("zoom around pixel should work");

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyBarSpacing));
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn scroll_to_position_registers_lwc_apply_right_offset_invalidation_kind() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine.clear_pending_invalidation();

    let changed = engine
        .scroll_time_to_position_bars(2.0)
        .expect("scroll to position should work");
    assert!(changed);

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn scroll_to_realtime_registers_lwc_apply_right_offset_invalidation_kind() {
    let mut engine = build_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("navigation behavior should be valid");
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine
        .scroll_time_to_position_bars(3.0)
        .expect("scroll to position should work");
    engine.clear_pending_invalidation();

    let changed = engine
        .scroll_time_to_realtime()
        .expect("scroll to realtime should work");
    assert!(changed);

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn wheel_pan_registers_lwc_apply_right_offset_invalidation_kind() {
    let mut engine = build_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("navigation behavior should be valid");
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine.clear_pending_invalidation();

    let applied = engine
        .wheel_pan_time_visible(120.0, 0.25)
        .expect("wheel pan should work");
    assert!(applied != 0.0);

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn touch_drag_pan_registers_lwc_apply_right_offset_invalidation_kind() {
    let mut engine = build_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("navigation behavior should be valid");
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine.clear_pending_invalidation();

    let applied = engine
        .touch_drag_pan_time_visible(20.0, 0.0)
        .expect("touch drag pan should work");
    assert!(applied != 0.0);

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn wheel_zoom_registers_lwc_bar_spacing_and_right_offset_without_apply_range() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
        DataPoint::new(4.0, 2.0),
    ]);
    engine.clear_pending_invalidation();

    let factor = engine
        .wheel_zoom_time_visible(-120.0, 300.0, 0.2, 1e-9)
        .expect("wheel zoom should work");
    assert!(factor > 1.0);

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyBarSpacing));
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
    assert!(!kinds.contains(&TimeScaleInvalidationType::ApplyRange));
}

#[test]
fn pinch_zoom_registers_lwc_bar_spacing_and_right_offset_without_apply_range() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
        DataPoint::new(4.0, 2.0),
    ]);
    engine.clear_pending_invalidation();

    let factor = engine
        .pinch_zoom_time_visible(1.15, 280.0, 1e-9)
        .expect("pinch zoom should work");
    assert!(factor > 1.0);

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyBarSpacing));
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
    assert!(!kinds.contains(&TimeScaleInvalidationType::ApplyRange));
}

#[test]
fn zoom_limit_behavior_registers_lwc_bar_spacing_and_right_offset_intent() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine.clear_pending_invalidation();

    engine
        .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
            min_bar_spacing_px: 0.5,
            max_bar_spacing_px: Some(2.0),
        })
        .expect("zoom limit behavior should be valid");

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyBarSpacing));
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn resize_behavior_with_viewport_change_registers_lwc_bar_spacing_and_right_offset_intent() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine
        .set_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: true,
            anchor: TimeScaleResizeAnchor::Right,
        })
        .expect("resize behavior should be valid");
    engine.clear_pending_invalidation();

    engine
        .set_viewport(Viewport::new(1200, 500))
        .expect("viewport resize should work");

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyBarSpacing));
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn navigation_behavior_with_right_offset_registers_lwc_apply_right_offset_intent() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine.clear_pending_invalidation();

    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .expect("navigation behavior should be valid");

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}

#[test]
fn navigation_behavior_with_bar_spacing_registers_lwc_bar_spacing_and_right_offset_intent() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 1.0),
        DataPoint::new(1.0, 2.0),
        DataPoint::new(2.0, 1.5),
        DataPoint::new(3.0, 2.2),
    ]);
    engine.clear_pending_invalidation();

    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 1.0,
            bar_spacing_px: Some(2.0),
        })
        .expect("navigation behavior should be valid");

    let pending = engine
        .lwc_pending_invalidation()
        .expect("lwc pending invalidation");
    let kinds = pending
        .time_scale_invalidations()
        .iter()
        .map(|invalidation| invalidation.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyBarSpacing));
    assert!(kinds.contains(&TimeScaleInvalidationType::ApplyRightOffset));
}
