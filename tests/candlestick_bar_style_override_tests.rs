use chart_rs::api::{CandlestickBarStyleOverride, ChartEngine, ChartEngineConfig, StyledOhlcBar};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::render::{CanvasLayerKind, Color, NullRenderer};

#[test]
fn per_bar_style_override_applies_body_wick_and_border_colors() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    let style = engine.render_style();

    let override_body = Color::rgb(0.94, 0.72, 0.18);
    let override_wick = Color::rgb(0.31, 0.43, 0.86);
    let override_border = Color::rgb(0.52, 0.24, 0.74);

    engine
        .set_styled_candles(vec![
            StyledOhlcBar::new(
                OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle with override"),
            )
            .with_style_override(CandlestickBarStyleOverride {
                color: Some(override_body),
                wick_color: Some(override_wick),
                border_color: Some(override_border),
            }),
            StyledOhlcBar::new(
                OhlcBar::new(40.0, 12.0, 14.0, 10.0, 10.5).expect("bear candle default"),
            ),
        ])
        .expect("set styled candles");

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

    assert!(series.lines.iter().any(|line| line.color == override_wick));
    assert!(series.rects.iter().any(|rect| {
        rect.fill_color == override_body
            && rect.border_color == override_border
            && rect.border_width > 0.0
    }));

    // Second candle has no per-bar override and must keep directional fallback.
    assert!(
        series
            .lines
            .iter()
            .any(|line| line.color == style.candlestick_wick_down_color)
    );
    assert!(
        series
            .rects
            .iter()
            .any(|rect| rect.fill_color == style.candlestick_down_color)
    );
}

#[test]
fn per_bar_style_override_uses_directional_fallback_for_missing_fields() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    let style = engine.render_style();
    let override_body = Color::rgb(0.93, 0.73, 0.21);

    engine
        .set_styled_candles(vec![
            StyledOhlcBar::new(OhlcBar::new(20.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"))
                .with_style_override(CandlestickBarStyleOverride {
                    color: Some(override_body),
                    wick_color: None,
                    border_color: None,
                }),
        ])
        .expect("set styled candles");

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

    assert!(series.rects.iter().any(|rect| {
        rect.fill_color == override_body && rect.border_color == style.candlestick_border_up_color
    }));
    assert!(
        series
            .lines
            .iter()
            .any(|line| line.color == style.candlestick_wick_up_color)
    );
}

#[test]
fn set_candles_and_update_candle_without_override_clear_previous_per_bar_styles() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let override_body = Color::rgb(0.88, 0.69, 0.26);
    engine
        .set_styled_candles(vec![
            StyledOhlcBar::new(OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"))
                .with_style_override(CandlestickBarStyleOverride {
                    color: Some(override_body),
                    wick_color: None,
                    border_color: None,
                }),
        ])
        .expect("set styled candles");

    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"),
    ]);
    engine
        .update_candle(OhlcBar::new(10.0, 10.0, 13.0, 9.0, 11.8).expect("replacement"))
        .expect("update candle");

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

    assert!(
        series
            .rects
            .iter()
            .all(|rect| rect.fill_color != override_body),
        "set_candles/update_candle without style payload must clear previous override"
    );
}

#[test]
fn append_and_update_styled_candle_replace_override_payload() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let initial_body_override = Color::rgb(0.83, 0.62, 0.24);
    let replacement_body_override = Color::rgb(0.28, 0.66, 0.84);

    engine
        .append_styled_candle(
            StyledOhlcBar::new(OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("append"))
                .with_style_override(CandlestickBarStyleOverride {
                    color: Some(initial_body_override),
                    wick_color: None,
                    border_color: None,
                }),
        )
        .expect("append styled candle");
    engine
        .update_styled_candle(
            StyledOhlcBar::new(OhlcBar::new(10.0, 10.0, 13.0, 9.0, 11.9).expect("replace"))
                .with_style_override(CandlestickBarStyleOverride {
                    color: Some(replacement_body_override),
                    wick_color: None,
                    border_color: None,
                }),
        )
        .expect("update styled candle");

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

    assert!(
        series
            .rects
            .iter()
            .any(|rect| rect.fill_color == replacement_body_override)
    );
    assert!(
        series
            .rects
            .iter()
            .all(|rect| rect.fill_color != initial_body_override),
        "updated styled candle should replace prior per-bar override"
    );
}
