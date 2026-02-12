use chart_rs::api::{ChartEngine, ChartEngineConfig, InteractionInputBehavior};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_interaction_input_behavior() {
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let behavior = config.interaction_input_behavior;
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
}

#[test]
fn chart_engine_config_applies_interaction_input_behavior() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_interaction_input_behavior(InteractionInputBehavior {
            handle_scroll: true,
            handle_scale: false,
            scroll_mouse_wheel: true,
            scroll_pressed_mouse_move: true,
            scroll_horz_touch_drag: true,
            scroll_vert_touch_drag: true,
            scale_mouse_wheel: true,
            scale_pinch: true,
            scale_axis_pressed_mouse_move: true,
            scale_axis_double_click_reset: true,
        });
    let renderer = NullRenderer::default();
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = engine.interaction_input_behavior();
    assert!(!behavior.handle_scale);

    let (start_before, end_before) = engine.time_visible_range();
    let factor = engine
        .wheel_zoom_time_visible(-120.0, 500.0, 0.2, 0.5)
        .expect("zoom path should be gated");
    assert!((factor - 1.0).abs() <= 1e-12);
    assert_eq!(engine.time_visible_range(), (start_before, end_before));
}

#[test]
fn chart_engine_config_json_without_interaction_behavior_uses_defaults() {
    let json = r#"{
  "viewport": { "width": 800, "height": 500 },
  "time_start": 0.0,
  "time_end": 10.0,
  "price_min": 0.0,
  "price_max": 100.0
}"#;
    let config = ChartEngineConfig::from_json_str(json).expect("parse config");
    let behavior = config.interaction_input_behavior;
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
}
