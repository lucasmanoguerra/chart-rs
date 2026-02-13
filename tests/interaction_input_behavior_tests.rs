use chart_rs::api::{
    ChartEngine, ChartEngineConfig, InteractionInputBehavior, TimeScaleNavigationBehavior,
};
use chart_rs::core::Viewport;
use chart_rs::interaction::InteractionMode;
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
fn interaction_input_behavior_defaults_to_enabled_scroll_and_scale() {
    let mut engine = build_engine();
    let behavior = engine.interaction_input_behavior();
    assert!(behavior.handle_scroll);
    assert!(behavior.handle_scale);
    assert!(behavior.scroll_mouse_wheel);
    assert!(behavior.scroll_pressed_mouse_move);
    assert!(behavior.scroll_horz_touch_drag);
    assert!(behavior.scroll_vert_touch_drag);
    assert!(behavior.scale_mouse_wheel);
    assert!(behavior.scale_pinch);
    assert!(behavior.scale_axis_pressed_mouse_move);
    assert!(behavior.scale_axis_double_click_reset);

    engine
        .wheel_pan_time_visible(120.0, 0.2)
        .expect("wheel pan should be enabled by default");
    let (start_after_pan, end_after_pan) = engine.time_visible_range();
    assert!((start_after_pan - 20.0).abs() <= 1e-9);
    assert!((end_after_pan - 120.0).abs() <= 1e-9);

    engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("wheel zoom should be enabled by default");
    let (start_after_zoom, end_after_zoom) = engine.time_visible_range();
    assert!((start_after_zoom - 28.333333333333336).abs() <= 1e-9);
    assert!((end_after_zoom - 111.66666666666667).abs() <= 1e-9);
}

#[test]
fn disabling_scroll_gates_drag_and_wheel_pan_paths_only() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        handle_scroll: false,
        handle_scale: true,
        ..InteractionInputBehavior::default()
    });

    let (initial_start, initial_end) = engine.time_visible_range();

    engine.pan_start();
    assert_eq!(engine.interaction_mode(), InteractionMode::Idle);

    engine
        .pan_time_visible_by_pixels(300.0)
        .expect("drag pan should be no-op when scroll is disabled");
    let delta = engine
        .wheel_pan_time_visible(120.0, 0.2)
        .expect("wheel pan should be no-op when scroll is disabled");
    assert!((delta - 0.0).abs() <= 1e-12);

    let (after_scroll_start, after_scroll_end) = engine.time_visible_range();
    assert!((after_scroll_start - initial_start).abs() <= 1e-12);
    assert!((after_scroll_end - initial_end).abs() <= 1e-12);

    // Scale path remains active.
    engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("wheel zoom should remain enabled");
    let (after_zoom_start, after_zoom_end) = engine.time_visible_range();
    assert!((after_zoom_start - initial_start).abs() > 1e-9);
    assert!((after_zoom_end - initial_end).abs() > 1e-9);
}

#[test]
fn disabling_scale_gates_wheel_and_pinch_zoom_paths_only() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        handle_scroll: true,
        handle_scale: false,
        ..InteractionInputBehavior::default()
    });

    let (initial_start, initial_end) = engine.time_visible_range();

    let wheel_factor = engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("wheel zoom should become no-op when scale is disabled");
    assert!((wheel_factor - 1.0).abs() <= 1e-12);

    let pinch_factor = engine
        .pinch_zoom_time_visible(1.8, 500.0, 0.5)
        .expect("pinch zoom should become no-op when scale is disabled");
    assert!((pinch_factor - 1.0).abs() <= 1e-12);

    let (after_scale_start, after_scale_end) = engine.time_visible_range();
    assert!((after_scale_start - initial_start).abs() <= 1e-12);
    assert!((after_scale_end - initial_end).abs() <= 1e-12);

    // Scroll path remains active.
    let pan_delta = engine
        .wheel_pan_time_visible(120.0, 0.2)
        .expect("wheel pan should remain enabled");
    assert!((pan_delta - 20.0).abs() <= 1e-9);
}

#[test]
fn disabled_paths_are_noop_without_validation_errors() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        handle_scroll: false,
        handle_scale: false,
        ..InteractionInputBehavior::default()
    });

    let pan_delta = engine
        .wheel_pan_time_visible(f64::NAN, f64::NAN)
        .expect("disabled wheel pan should bypass validation and no-op");
    assert!((pan_delta - 0.0).abs() <= 1e-12);

    let zoom_factor = engine
        .wheel_zoom_time_visible(f64::NAN, f64::NAN, f64::NAN, f64::NAN)
        .expect("disabled wheel zoom should bypass validation and no-op");
    assert!((zoom_factor - 1.0).abs() <= 1e-12);

    let pinch_factor = engine
        .pinch_zoom_time_visible(f64::NAN, f64::NAN, f64::NAN)
        .expect("disabled pinch zoom should bypass validation and no-op");
    assert!((pinch_factor - 1.0).abs() <= 1e-12);
}

#[test]
fn pointer_crosshair_flow_remains_active_when_scroll_and_scale_are_disabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        handle_scroll: false,
        handle_scale: false,
        ..InteractionInputBehavior::default()
    });

    engine.pointer_move(123.0, 77.0);
    let crosshair = engine.crosshair_state();
    assert!(crosshair.visible);
    assert!((crosshair.x - 123.0).abs() <= 1e-9);
    assert!((crosshair.y - 77.0).abs() <= 1e-9);
}

#[test]
fn disabling_scroll_mouse_wheel_keeps_drag_pan_enabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scroll_mouse_wheel: false,
        ..InteractionInputBehavior::default()
    });

    engine.pan_start();
    assert_eq!(engine.interaction_mode(), InteractionMode::Panning);
    engine.pan_end();
    assert_eq!(engine.interaction_mode(), InteractionMode::Idle);

    let delta = engine
        .wheel_pan_time_visible(120.0, 0.2)
        .expect("wheel pan should be disabled");
    assert!((delta - 0.0).abs() <= 1e-12);
}

#[test]
fn disabling_scroll_pressed_mouse_move_keeps_wheel_pan_enabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scroll_pressed_mouse_move: false,
        ..InteractionInputBehavior::default()
    });

    engine.pan_start();
    assert_eq!(engine.interaction_mode(), InteractionMode::Idle);

    let delta = engine
        .wheel_pan_time_visible(120.0, 0.2)
        .expect("wheel pan should remain enabled");
    assert!((delta - 20.0).abs() <= 1e-9);
}

#[test]
fn disabling_scale_mouse_wheel_keeps_pinch_zoom_enabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scale_mouse_wheel: false,
        ..InteractionInputBehavior::default()
    });

    let (initial_start, initial_end) = engine.time_visible_range();
    let wheel_factor = engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("wheel zoom should be disabled");
    assert!((wheel_factor - 1.0).abs() <= 1e-12);

    let pinch_factor = engine
        .pinch_zoom_time_visible(1.8, 500.0, 0.5)
        .expect("pinch zoom should remain enabled");
    assert!((pinch_factor - 1.8).abs() <= 1e-12);
    let (after_start, after_end) = engine.time_visible_range();
    assert!((after_start - initial_start).abs() > 1e-9);
    assert!((after_end - initial_end).abs() > 1e-9);
}

#[test]
fn disabling_scale_pinch_keeps_wheel_zoom_enabled() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scale_pinch: false,
        ..InteractionInputBehavior::default()
    });

    let (initial_start, initial_end) = engine.time_visible_range();
    let pinch_factor = engine
        .pinch_zoom_time_visible(1.8, 500.0, 0.5)
        .expect("pinch zoom should be disabled");
    assert!((pinch_factor - 1.0).abs() <= 1e-12);

    let wheel_factor = engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("wheel zoom should remain enabled");
    assert!((wheel_factor - 1.2).abs() <= 1e-12);
    let (after_start, after_end) = engine.time_visible_range();
    assert!((after_start - initial_start).abs() > 1e-9);
    assert!((after_end - initial_end).abs() > 1e-9);
}

#[test]
fn axis_drag_scale_and_double_click_reset_are_enabled_by_default() {
    let mut engine = build_engine();
    let before_span = {
        let domain = engine.price_domain();
        (domain.1 - domain.0).abs()
    };

    let factor = engine
        .axis_drag_scale_price(120.0, 250.0, 0.2, 1e-6)
        .expect("axis drag scale should be enabled by default");
    assert!((factor - 1.2).abs() <= 1e-12);

    let after_span = {
        let domain = engine.price_domain();
        (domain.1 - domain.0).abs()
    };
    assert!(after_span > before_span);

    engine.set_price_scale_realtime_behavior(chart_rs::api::PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: false,
    });
    engine.set_data(vec![
        chart_rs::core::DataPoint::new(0.0, 10.0),
        chart_rs::core::DataPoint::new(1.0, 20.0),
    ]);
    let changed = engine
        .axis_double_click_reset_price_scale()
        .expect("axis reset should be enabled by default");
    assert!(changed);

    engine
        .set_time_visible_range(30.0, 60.0)
        .expect("set constrained time visible range");
    let time_changed = engine
        .axis_double_click_reset_time_scale()
        .expect("time axis reset should be enabled by default");
    assert!(time_changed);
    assert_eq!(engine.time_visible_range(), engine.time_full_range());
}

#[test]
fn disabling_scale_master_gate_disables_axis_paths() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        handle_scale: false,
        ..InteractionInputBehavior::default()
    });

    let before = engine.price_domain();
    let factor = engine
        .axis_drag_scale_price(f64::NAN, f64::NAN, f64::NAN, f64::NAN)
        .expect("disabled axis drag should bypass validation and no-op");
    assert!((factor - 1.0).abs() <= 1e-12);
    let changed = engine
        .axis_double_click_reset_price_scale()
        .expect("disabled axis reset should no-op");
    assert!(!changed);
    let time_changed = engine
        .axis_double_click_reset_time_scale()
        .expect("disabled time axis reset should no-op");
    assert!(!time_changed);
    assert_eq!(engine.price_domain(), before);
}

#[test]
fn disabling_axis_specific_gates_disables_only_axis_paths() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scale_axis_pressed_mouse_move: false,
        scale_axis_double_click_reset: false,
        ..InteractionInputBehavior::default()
    });

    let before = engine.price_domain();
    let factor = engine
        .axis_drag_scale_price(120.0, 250.0, 0.2, 1e-6)
        .expect("axis drag should be gated");
    assert!((factor - 1.0).abs() <= 1e-12);
    let changed = engine
        .axis_double_click_reset_price_scale()
        .expect("axis reset should be gated");
    assert!(!changed);
    let time_changed = engine
        .axis_double_click_reset_time_scale()
        .expect("time axis reset should be gated");
    assert!(!time_changed);
    assert_eq!(engine.price_domain(), before);

    // Non-axis scale path remains enabled.
    let wheel_factor = engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("wheel zoom should remain enabled");
    assert!((wheel_factor - 1.2).abs() <= 1e-12);
}
