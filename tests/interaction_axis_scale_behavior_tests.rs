use chart_rs::api::{
    ChartEngine, ChartEngineConfig, InteractionInputBehavior, PriceScaleRealtimeBehavior,
    TimeScaleNavigationBehavior, TimeScaleScrollZoomBehavior,
};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn axis_drag_scale_price_changes_domain_span_when_enabled() {
    let mut engine = build_engine();
    let before = engine.price_domain();
    let before_span = (before.1 - before.0).abs();

    let zoom_out_factor = engine
        .axis_drag_scale_price(120.0, 250.0, 0.2, 1e-6)
        .expect("axis drag zoom out");
    assert!((zoom_out_factor - 1.2).abs() <= 1e-12);
    let zoomed_out_span = {
        let domain = engine.price_domain();
        (domain.1 - domain.0).abs()
    };
    assert!(zoomed_out_span > before_span);

    let zoom_in_factor = engine
        .axis_drag_scale_price(-120.0, 250.0, 0.2, 1e-6)
        .expect("axis drag zoom in");
    assert!((zoom_in_factor - (1.0 / 1.2)).abs() <= 1e-12);
    let zoomed_in_span = {
        let domain = engine.price_domain();
        (domain.1 - domain.0).abs()
    };
    assert!(zoomed_in_span < zoomed_out_span);
}

#[test]
fn axis_drag_pan_price_preserves_anchor_projection_and_span_when_enabled() {
    let mut engine = build_engine();
    let before = engine.price_domain();
    let before_span = (before.1 - before.0).abs();
    let anchor_y = 250.0;
    let drag_delta = 80.0;
    let anchor_price_before = engine.map_pixel_to_price(anchor_y).expect("anchor price");

    let changed = engine
        .axis_drag_pan_price(drag_delta, anchor_y)
        .expect("axis drag pan");
    assert!(changed);

    let after = engine.price_domain();
    let after_span = (after.1 - after.0).abs();
    assert!((after_span - before_span).abs() <= 1e-9);

    let shifted_anchor_price_after = engine
        .map_pixel_to_price(anchor_y + drag_delta)
        .expect("shifted anchor price");
    assert!((anchor_price_before - shifted_anchor_price_after).abs() <= 1e-6);
}

#[test]
fn axis_drag_pan_price_is_stable_under_repeated_drags() {
    let mut engine = build_engine();
    let initial_span = {
        let domain = engine.price_domain();
        (domain.1 - domain.0).abs()
    };

    for step in 0..200 {
        let delta = if step % 2 == 0 { 18.0 } else { -16.0 };
        let changed = engine
            .axis_drag_pan_price(delta, 220.0)
            .expect("axis drag pan step");
        assert!(changed);
        let domain = engine.price_domain();
        assert!(domain.0.is_finite() && domain.1.is_finite());
        assert!(domain.1 > domain.0);
        let span = (domain.1 - domain.0).abs();
        assert!((span - initial_span).abs() <= 1e-6);
    }
}

#[test]
fn axis_drag_pan_price_rejects_invalid_input_when_enabled() {
    let mut engine = build_engine();
    let err = engine
        .axis_drag_pan_price(f64::NAN, 120.0)
        .expect_err("nan pan delta must fail");
    assert!(err.to_string().contains("axis drag pan delta"));
}

#[test]
fn axis_drag_scale_time_changes_visible_span_when_enabled() {
    let mut engine = build_engine();
    let before = engine.time_visible_range();
    let before_span = (before.1 - before.0).abs();

    let zoom_in_factor = engine
        .axis_drag_scale_time(120.0, 500.0, 0.2, 1e-6)
        .expect("axis drag time zoom in");
    assert!((zoom_in_factor - 1.2).abs() <= 1e-12);
    let zoomed_in_span = {
        let range = engine.time_visible_range();
        (range.1 - range.0).abs()
    };
    assert!(zoomed_in_span < before_span);

    let zoom_out_factor = engine
        .axis_drag_scale_time(-120.0, 500.0, 0.2, 1e-6)
        .expect("axis drag time zoom out");
    assert!((zoom_out_factor - (1.0 / 1.2)).abs() <= 1e-12);
    let zoomed_out_span = {
        let range = engine.time_visible_range();
        (range.1 - range.0).abs()
    };
    assert!(zoomed_out_span > zoomed_in_span);
}

#[test]
fn axis_drag_scale_time_with_right_bar_stays_and_right_offset_px_preserves_right_margin_px() {
    let mut engine = build_engine();
    engine
        .set_time_scale_right_offset_px(Some(120.0))
        .expect("set right offset px");
    engine
        .set_time_scale_scroll_zoom_behavior(TimeScaleScrollZoomBehavior {
            right_bar_stays_on_scroll: true,
        })
        .expect("set scroll zoom behavior");

    engine
        .axis_drag_scale_time(120.0, 500.0, 0.2, 1e-6)
        .expect("axis drag time zoom");

    let (_, full_end) = engine.time_full_range();
    let (start_after, end_after) = engine.time_visible_range();
    let span_after = end_after - start_after;
    let expected_offset = span_after * (120.0 / 1000.0);
    assert!(((end_after - full_end) - expected_offset).abs() <= 1e-9);
}

#[test]
fn axis_double_click_reset_time_scale_restores_full_range_when_enabled() {
    let mut engine = build_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("disable default spacing navigation");
    engine
        .set_time_visible_range(20.0, 40.0)
        .expect("set constrained time visible range");

    let changed = engine
        .axis_double_click_reset_time_scale()
        .expect("time axis reset");
    assert!(changed);
    assert_eq!(engine.time_visible_range(), engine.time_full_range());
}

#[test]
fn axis_double_click_reset_time_scale_is_noop_when_already_full() {
    let mut engine = build_engine();
    engine
        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 0.0,
            bar_spacing_px: None,
        })
        .expect("disable default spacing navigation");

    let changed = engine
        .axis_double_click_reset_time_scale()
        .expect("time axis reset");
    assert!(!changed);
    assert_eq!(engine.time_visible_range(), engine.time_full_range());
}

#[test]
fn disabling_scale_master_gate_disables_axis_drag_and_reset_paths() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        handle_scale: false,
        ..InteractionInputBehavior::default()
    });

    let before = engine.price_domain();
    let factor = engine
        .axis_drag_scale_price(f64::NAN, f64::NAN, f64::NAN, f64::NAN)
        .expect("disabled axis drag should no-op");
    assert!((factor - 1.0).abs() <= 1e-12);
    let pan_changed = engine
        .axis_drag_pan_price(f64::NAN, f64::NAN)
        .expect("disabled axis pan should no-op");
    assert!(!pan_changed);
    let time_factor = engine
        .axis_drag_scale_time(f64::NAN, f64::NAN, f64::NAN, f64::NAN)
        .expect("disabled time axis drag should no-op");
    assert!((time_factor - 1.0).abs() <= 1e-12);
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
fn disabling_axis_specific_flags_disables_only_axis_paths() {
    let mut engine = build_engine();
    engine.set_interaction_input_behavior(InteractionInputBehavior {
        scale_axis_pressed_mouse_move: false,
        scale_axis_double_click_reset: false,
        ..InteractionInputBehavior::default()
    });
    let before = engine.price_domain();

    let factor = engine
        .axis_drag_scale_price(120.0, 250.0, 0.2, 1e-6)
        .expect("disabled axis drag should no-op");
    assert!((factor - 1.0).abs() <= 1e-12);
    let pan_changed = engine
        .axis_drag_pan_price(120.0, 250.0)
        .expect("disabled axis pan should no-op");
    assert!(!pan_changed);
    let time_factor = engine
        .axis_drag_scale_time(120.0, 500.0, 0.2, 1e-6)
        .expect("disabled time axis drag should no-op");
    assert!((time_factor - 1.0).abs() <= 1e-12);
    assert_eq!(engine.price_domain(), before);

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
fn axis_double_click_reset_autoscales_points_when_enabled() {
    let mut engine = build_engine();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: false,
    });
    engine.set_data(vec![DataPoint::new(0.0, 10.0), DataPoint::new(1.0, 20.0)]);
    let before = engine.price_domain();

    let changed = engine
        .axis_double_click_reset_price_scale()
        .expect("axis reset");
    assert!(changed);

    let after = engine.price_domain();
    assert!((after.0 - before.0).abs() > 1e-9 || (after.1 - before.1).abs() > 1e-9);
    assert!(after.0 < 10.0);
    assert!(after.1 > 20.0);
}

#[test]
fn axis_double_click_reset_prioritizes_candle_domain_when_available() {
    let mut engine = build_engine();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: false,
    });

    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(1.0, 1_000.0),
    ]);
    engine.set_candles(vec![
        OhlcBar::new(0.0, 45.0, 60.0, 40.0, 55.0).expect("candle"),
        OhlcBar::new(1.0, 55.0, 62.0, 50.0, 58.0).expect("candle"),
    ]);

    let changed = engine
        .axis_double_click_reset_price_scale()
        .expect("axis reset");
    assert!(changed);
    let after = engine.price_domain();
    assert!(after.1 < 200.0);
}
