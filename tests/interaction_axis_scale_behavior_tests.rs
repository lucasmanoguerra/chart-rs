use chart_rs::api::{
    ChartEngine, ChartEngineConfig, InteractionInputBehavior, PriceScaleRealtimeBehavior,
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
    let time_factor = engine
        .axis_drag_scale_time(f64::NAN, f64::NAN, f64::NAN, f64::NAN)
        .expect("disabled time axis drag should no-op");
    assert!((time_factor - 1.0).abs() <= 1e-12);
    let changed = engine
        .axis_double_click_reset_price_scale()
        .expect("disabled axis reset should no-op");
    assert!(!changed);
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
    let time_factor = engine
        .axis_drag_scale_time(120.0, 500.0, 0.2, 1e-6)
        .expect("disabled time axis drag should no-op");
    assert!((time_factor - 1.0).abs() <= 1e-12);
    assert_eq!(engine.price_domain(), before);

    let changed = engine
        .axis_double_click_reset_price_scale()
        .expect("disabled axis reset should no-op");
    assert!(!changed);
    assert_eq!(engine.price_domain(), before);
}

#[test]
fn axis_double_click_reset_autoscales_points_when_enabled() {
    let mut engine = build_engine();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: false,
        autoscale_on_time_range_change: false,
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
        autoscale_on_time_range_change: false,
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
