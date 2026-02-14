use chart_rs::api::{
    CandlestickBodyMode, CandlestickStyleBehavior, ChartEngine, ChartEngineConfig,
};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::render::{CanvasLayerKind, Color, NullRenderer};

#[test]
fn candlestick_style_behavior_defaults_match_render_style_defaults() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.candlestick_style_behavior(),
        CandlestickStyleBehavior::default()
    );
}

#[test]
fn set_candlestick_style_behavior_updates_render_style_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = CandlestickStyleBehavior {
        up_color: Color::rgb(0.11, 0.71, 0.43),
        down_color: Color::rgb(0.81, 0.18, 0.21),
        wick_color: None,
        wick_up_color: Color::rgb(0.06, 0.52, 0.30),
        wick_down_color: Color::rgb(0.58, 0.11, 0.15),
        border_color: None,
        border_up_color: Color::rgb(0.03, 0.40, 0.24),
        border_down_color: Color::rgb(0.43, 0.08, 0.11),
        body_mode: CandlestickBodyMode::HollowUp,
        wick_width_px: 2.0,
        border_width_px: 1.5,
        show_wicks: true,
        show_borders: true,
    };
    engine
        .set_candlestick_style_behavior(behavior)
        .expect("set behavior");

    assert_eq!(engine.candlestick_style_behavior(), behavior);
    let style = engine.render_style();
    assert_eq!(style.candlestick_up_color, behavior.up_color);
    assert_eq!(style.candlestick_down_color, behavior.down_color);
    assert_eq!(style.candlestick_wick_up_color, behavior.wick_up_color);
    assert_eq!(style.candlestick_wick_down_color, behavior.wick_down_color);
    assert_eq!(style.candlestick_border_up_color, behavior.border_up_color);
    assert_eq!(
        style.candlestick_border_down_color,
        behavior.border_down_color
    );
    assert_eq!(style.candlestick_body_mode, behavior.body_mode);
    assert!((style.candlestick_wick_width_px - behavior.wick_width_px).abs() <= 1e-9);
    assert!((style.candlestick_border_width_px - behavior.border_width_px).abs() <= 1e-9);
    assert_eq!(style.show_candlestick_wicks, behavior.show_wicks);
    assert_eq!(style.show_candlestick_borders, behavior.show_borders);
}

#[test]
fn candlestick_style_behavior_is_applied_to_rendered_primitives() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"),
        OhlcBar::new(30.0, 12.0, 14.0, 10.0, 10.5).expect("bear candle"),
    ]);

    let behavior = CandlestickStyleBehavior {
        up_color: Color::rgb(0.16, 0.66, 0.49),
        down_color: Color::rgb(0.86, 0.26, 0.22),
        wick_color: None,
        wick_up_color: Color::rgb(0.12, 0.53, 0.38),
        wick_down_color: Color::rgb(0.63, 0.16, 0.13),
        border_color: None,
        border_up_color: Color::rgb(0.10, 0.42, 0.30),
        border_down_color: Color::rgb(0.50, 0.12, 0.10),
        body_mode: CandlestickBodyMode::HollowUp,
        wick_width_px: 2.0,
        border_width_px: 2.0,
        show_wicks: false,
        show_borders: false,
    };
    engine
        .set_candlestick_style_behavior(behavior)
        .expect("set behavior");

    let layered = engine
        .build_layered_render_frame()
        .expect("build layered render frame");
    let main = layered
        .panes
        .iter()
        .find(|pane| pane.pane_id == engine.main_pane_id())
        .expect("main pane");
    let series = main
        .layers
        .iter()
        .find(|layer| layer.kind == CanvasLayerKind::Series)
        .expect("series layer");

    assert!(series.lines.is_empty(), "wicks must be hidden");
    assert!(
        series.rects.len() >= 2,
        "candlestick bodies should still render"
    );
    assert!(series.rects.iter().all(|rect| rect.border_width == 0.0));
    assert!(
        series
            .rects
            .iter()
            .any(|rect| (rect.fill_color.alpha - 0.0).abs() <= 1e-12)
    );
    assert!(
        series
            .rects
            .iter()
            .any(|rect| rect.fill_color == behavior.down_color)
    );
}

#[test]
fn candlestick_style_behavior_shared_wick_and_border_colors_override_up_down_fields() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = CandlestickStyleBehavior {
        up_color: Color::rgb(0.16, 0.66, 0.49),
        down_color: Color::rgb(0.86, 0.26, 0.22),
        wick_color: Some(Color::rgb(0.41, 0.33, 0.21)),
        wick_up_color: Color::rgb(0.12, 0.53, 0.38),
        wick_down_color: Color::rgb(0.63, 0.16, 0.13),
        border_color: Some(Color::rgb(0.24, 0.29, 0.46)),
        border_up_color: Color::rgb(0.10, 0.42, 0.30),
        border_down_color: Color::rgb(0.50, 0.12, 0.10),
        body_mode: CandlestickBodyMode::Solid,
        wick_width_px: 2.0,
        border_width_px: 1.0,
        show_wicks: true,
        show_borders: true,
    };
    engine
        .set_candlestick_style_behavior(behavior)
        .expect("set behavior");

    let style = engine.render_style();
    assert_eq!(
        style.candlestick_wick_up_color,
        behavior.wick_color.expect("wick")
    );
    assert_eq!(
        style.candlestick_wick_down_color,
        behavior.wick_color.expect("wick")
    );
    assert_eq!(
        style.candlestick_border_up_color,
        behavior.border_color.expect("border")
    );
    assert_eq!(
        style.candlestick_border_down_color,
        behavior.border_color.expect("border")
    );
}
