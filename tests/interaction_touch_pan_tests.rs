use chart_rs::api::{
    ChartEngine, ChartEngineConfig, InteractionInputBehavior, TimeScaleNavigationBehavior,
};
use chart_rs::core::Viewport;
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

#[test]
fn touch_drag_defaults_to_enabled_horizontal_and_vertical_paths() {
    let mut engine = build_engine();
    let delta = engine
        .touch_drag_pan_time_visible(120.0, 20.0)
        .expect("touch drag pan");
    assert!((delta + 12.0).abs() <= 1e-9);

    let (start, end) = engine.time_visible_range();
    assert!((start + 12.0).abs() <= 1e-9);
    assert!((end - 88.0).abs() <= 1e-9);
}

#[test]
fn disabling_touch_drag_flags_makes_touch_pan_noop() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scroll_horz_touch_drag: false,
        scroll_vert_touch_drag: false,
        ..InteractionInputBehavior::default()
    });

    let before = engine.time_visible_range();
    let delta = engine
        .touch_drag_pan_time_visible(120.0, 40.0)
        .expect("touch pan no-op");
    assert!((delta - 0.0).abs() <= 1e-12);
    assert_eq!(engine.time_visible_range(), before);
}

#[test]
fn vertical_touch_drag_path_can_drive_pan_when_horizontal_is_disabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scroll_horz_touch_drag: false,
        scroll_vert_touch_drag: true,
        ..InteractionInputBehavior::default()
    });

    let delta = engine
        .touch_drag_pan_time_visible(20.0, -50.0)
        .expect("touch pan via vertical axis");
    assert!((delta - 10.0).abs() <= 1e-9);

    let (start, end) = engine.time_visible_range();
    assert!((start - 10.0).abs() <= 1e-9);
    assert!((end - 110.0).abs() <= 1e-9);
}

#[test]
fn touch_pan_rejects_invalid_input_when_enabled() {
    let mut engine = build_engine();
    let err = engine
        .touch_drag_pan_time_visible(f64::NAN, 10.0)
        .expect_err("nan touch drag must fail");
    assert!(matches!(err, chart_rs::ChartError::InvalidData(_)));
}

#[test]
fn vertical_touch_pan_ignores_horizontal_nan_when_horizontal_path_is_disabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scroll_horz_touch_drag: false,
        scroll_vert_touch_drag: true,
        ..InteractionInputBehavior::default()
    });

    let delta = engine
        .touch_drag_pan_time_visible(f64::NAN, -25.0)
        .expect("horizontal delta must be ignored when disabled");
    assert!((delta - 5.0).abs() <= 1e-9);
}

#[test]
fn horizontal_touch_pan_ignores_vertical_nan_when_vertical_path_is_disabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scroll_horz_touch_drag: true,
        scroll_vert_touch_drag: false,
        ..InteractionInputBehavior::default()
    });

    let delta = engine
        .touch_drag_pan_time_visible(120.0, f64::NAN)
        .expect("vertical delta must be ignored when disabled");
    assert!((delta + 12.0).abs() <= 1e-9);
}

#[test]
fn touch_pan_rejects_non_finite_vertical_delta_when_vertical_path_is_enabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scroll_horz_touch_drag: false,
        scroll_vert_touch_drag: true,
        ..InteractionInputBehavior::default()
    });

    let err = engine
        .touch_drag_pan_time_visible(10.0, f64::NAN)
        .expect_err("vertical nan must fail");
    assert!(matches!(err, chart_rs::ChartError::InvalidData(_)));
}

#[test]
fn disabled_touch_pan_bypasses_validation_and_noops() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        handle_scroll: false,
        ..InteractionInputBehavior::default()
    });
    let delta = engine
        .touch_drag_pan_time_visible(f64::NAN, f64::NAN)
        .expect("disabled touch pan bypasses validation");
    assert!((delta - 0.0).abs() <= 1e-12);
}
