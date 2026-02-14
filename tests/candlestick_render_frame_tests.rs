use chart_rs::api::{CandlestickBodyMode, ChartEngine, ChartEngineConfig, RenderStyle};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::render::{CanvasLayerKind, Color, NullRenderer};

#[test]
fn build_render_frame_materializes_candlestick_wicks_and_bodies() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("candle 1"),
        OhlcBar::new(30.0, 12.0, 14.0, 11.0, 11.5).expect("candle 2"),
        OhlcBar::new(70.0, 11.5, 15.0, 10.5, 14.0).expect("candle 3"),
    ]);

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
        series.lines.len() >= 3,
        "expected at least one wick line per candle"
    );
    assert!(
        series.rects.len() >= 3,
        "expected at least one body rect per candle"
    );
}

#[test]
fn candlestick_render_uses_dedicated_candle_style_fields() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"),
        OhlcBar::new(30.0, 12.0, 14.0, 10.0, 10.5).expect("bear candle"),
    ]);

    let style = RenderStyle {
        last_price_up_color: Color::rgb(0.05, 0.05, 0.05),
        last_price_down_color: Color::rgb(0.95, 0.95, 0.95),
        candlestick_up_color: Color::rgb(0.11, 0.71, 0.43),
        candlestick_down_color: Color::rgb(0.81, 0.18, 0.21),
        candlestick_wick_up_color: Color::rgb(0.06, 0.52, 0.30),
        candlestick_wick_down_color: Color::rgb(0.58, 0.11, 0.15),
        candlestick_border_up_color: Color::rgb(0.03, 0.40, 0.24),
        candlestick_border_down_color: Color::rgb(0.43, 0.08, 0.11),
        candlestick_wick_width_px: 2.75,
        candlestick_border_width_px: 1.5,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

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

    let up_wicks: Vec<_> = series
        .lines
        .iter()
        .filter(|line| line.color == style.candlestick_wick_up_color)
        .collect();
    let down_wicks: Vec<_> = series
        .lines
        .iter()
        .filter(|line| line.color == style.candlestick_wick_down_color)
        .collect();
    assert!(
        !up_wicks.is_empty(),
        "expected bullish wick color from dedicated candlestick style field"
    );
    assert!(
        !down_wicks.is_empty(),
        "expected bearish wick color from dedicated candlestick style field"
    );
    assert!(up_wicks.iter().chain(down_wicks.iter()).all(|line| {
        line.stroke_width >= 1.0
            && line.stroke_width <= style.candlestick_wick_width_px
            && line.stroke_width.fract().abs() <= 1e-9
    }));
    assert!(series.rects.iter().any(|rect| {
        rect.fill_color == style.candlestick_up_color
            && rect.border_color == style.candlestick_border_up_color
            && rect.border_width > 0.0
            && rect.border_width <= style.candlestick_border_width_px
    }));
    assert!(series.rects.iter().any(|rect| {
        rect.fill_color == style.candlestick_down_color
            && rect.border_color == style.candlestick_border_down_color
            && rect.border_width > 0.0
            && rect.border_width <= style.candlestick_border_width_px
    }));
}

#[test]
fn candlestick_hollow_up_mode_makes_bull_body_transparent() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"),
        OhlcBar::new(30.0, 12.0, 14.0, 10.0, 10.5).expect("bear candle"),
    ]);

    let style = RenderStyle {
        candlestick_body_mode: CandlestickBodyMode::HollowUp,
        candlestick_up_color: Color::rgb(0.18, 0.70, 0.48),
        candlestick_down_color: Color::rgb(0.79, 0.19, 0.24),
        candlestick_border_up_color: Color::rgb(0.12, 0.58, 0.39),
        candlestick_border_down_color: Color::rgb(0.58, 0.12, 0.17),
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

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
        rect.border_color == style.candlestick_border_up_color
            && (rect.fill_color.alpha - 0.0).abs() <= 1e-12
    }));
    assert!(series.rects.iter().any(|rect| {
        rect.border_color == style.candlestick_border_down_color
            && rect.fill_color == style.candlestick_down_color
    }));
}

#[test]
fn candlestick_wick_visibility_toggle_is_applied() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"),
        OhlcBar::new(30.0, 12.0, 14.0, 10.0, 10.5).expect("bear candle"),
    ]);

    let style = RenderStyle {
        show_candlestick_wicks: false,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

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
        series.lines.is_empty(),
        "wick lines should be hidden when show_candlestick_wicks=false"
    );
    assert!(
        !series.rects.is_empty(),
        "candlestick bodies should still render when wicks are hidden"
    );
}

#[test]
fn candlestick_border_visibility_toggle_is_applied() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("bull candle"),
        OhlcBar::new(30.0, 12.0, 14.0, 10.0, 10.5).expect("bear candle"),
    ]);

    let style = RenderStyle {
        candlestick_border_width_px: 2.0,
        show_candlestick_borders: false,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

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
        !series.rects.is_empty(),
        "candlestick bodies should render when borders are hidden"
    );
    assert!(series.rects.iter().all(|rect| rect.border_width == 0.0));
}

#[test]
fn candlestick_body_width_follows_lightweight_optimal_formula_without_legacy_clamp() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(920, 460), 0.0, 100.0).with_price_domain(60.0, 320.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(0.0, 70.0, 86.0, 64.0, 82.0).expect("c1"),
        OhlcBar::new(10.0, 82.0, 95.0, 75.0, 90.0).expect("c2"),
        OhlcBar::new(20.0, 90.0, 108.0, 84.0, 101.0).expect("c3"),
        OhlcBar::new(30.0, 101.0, 122.0, 95.0, 115.0).expect("c4"),
        OhlcBar::new(40.0, 115.0, 136.0, 108.0, 128.0).expect("c5"),
        OhlcBar::new(50.0, 128.0, 156.0, 121.0, 145.0).expect("c6"),
        OhlcBar::new(60.0, 145.0, 178.0, 138.0, 168.0).expect("c7"),
        OhlcBar::new(70.0, 168.0, 206.0, 160.0, 194.0).expect("c8"),
        OhlcBar::new(80.0, 194.0, 238.0, 185.0, 225.0).expect("c9"),
        OhlcBar::new(90.0, 225.0, 272.0, 214.0, 258.0).expect("c10"),
        OhlcBar::new(100.0, 258.0, 306.0, 246.0, 292.0).expect("c11"),
    ]);

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

    let max_width = series
        .rects
        .iter()
        .map(|rect| rect.width)
        .reduce(f64::max)
        .expect("at least one candle body");

    let style = engine.render_style();
    let plot_right = (f64::from(engine.viewport().width) - style.price_axis_width_px)
        .clamp(0.0, f64::from(engine.viewport().width));
    let bar_spacing = plot_right / 10.0;
    let mut expected = lwc_optimal_candlestick_width(bar_spacing, 1.0);
    if expected >= 2.0 && ((expected as i64) % 2 == 0) {
        expected -= 1.0;
    }

    assert!(
        max_width > 24.0,
        "body width should not be constrained by the legacy 24px clamp"
    );
    assert!(
        (max_width - expected).abs() <= 1.0,
        "expected width near Lightweight formula; got {max_width}, expected {expected}"
    );
}

fn lwc_optimal_candlestick_width(bar_spacing: f64, pixel_ratio: f64) -> f64 {
    let special_from = 2.5;
    let special_to = 4.0;
    let special_coeff = 3.0;
    if bar_spacing >= special_from && bar_spacing <= special_to {
        return (special_coeff * pixel_ratio).floor();
    }

    let reducing_coeff = 0.2;
    let coeff = 1.0
        - reducing_coeff * (bar_spacing.max(special_to) - special_to).atan()
            / (std::f64::consts::PI * 0.5);
    let res = (bar_spacing * coeff * pixel_ratio).floor();
    let scaled_spacing = (bar_spacing * pixel_ratio).floor();
    res.min(scaled_spacing).max(pixel_ratio.floor())
}

#[test]
fn candlestick_border_only_body_path_is_used_for_tiny_bar_width() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(920, 460), 0.0, 10_000.0)
        .with_price_domain(60.0, 320.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(0.0, 70.0, 86.0, 64.0, 82.0).expect("bull candle"),
        OhlcBar::new(10.0, 82.0, 95.0, 75.0, 79.0).expect("bear candle"),
    ]);
    let style = RenderStyle {
        show_candlestick_wicks: false,
        show_candlestick_borders: true,
        candlestick_border_width_px: 1.0,
        candlestick_border_up_color: Color::rgb(0.07, 0.43, 0.29),
        candlestick_border_down_color: Color::rgb(0.50, 0.13, 0.11),
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

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

    assert!(!series.rects.is_empty(), "expected candle bodies");
    assert!(
        series.rects.iter().all(|rect| rect.width <= 1.1),
        "expected narrow bar width to trigger border-only path"
    );
    assert!(
        series.rects.iter().all(|rect| rect.border_width == 0.0),
        "border-only path should emit fill-only rects"
    );
    assert!(
        series
            .rects
            .iter()
            .any(|rect| rect.fill_color == style.candlestick_border_up_color)
    );
    assert!(
        series
            .rects
            .iter()
            .any(|rect| rect.fill_color == style.candlestick_border_down_color)
    );
}

#[test]
fn candlestick_dense_spacing_avoids_horizontal_overlap_with_prev_edge_policy() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(340, 260), 0.0, 100.0).with_price_domain(80.0, 140.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    let mut candles = Vec::new();
    for time in 40..=60 {
        let base = 95.0 + (time as f64 - 40.0) * 1.1;
        candles.push(
            OhlcBar::new(time as f64, base, base + 3.0, base - 3.0, base + 1.0)
                .expect("dense candle"),
        );
    }
    let candle_count = candles.len();
    engine.set_candles(candles);
    engine
        .set_render_style(RenderStyle {
            candlestick_wick_width_px: 3.0,
            candlestick_border_width_px: 2.0,
            show_candlestick_wicks: true,
            show_candlestick_borders: true,
            ..engine.render_style()
        })
        .expect("set style");

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

    let mut body_bounds: Vec<(i64, i64)> = series
        .rects
        .iter()
        .map(|rect| {
            let left = rect.x.round() as i64;
            let width = rect.width.round().max(1.0) as i64;
            let right = left + width - 1;
            (left, right)
        })
        .collect();
    body_bounds.sort_by_key(|(left, _)| *left);
    assert_eq!(
        body_bounds.len(),
        candle_count,
        "expected one body per candle"
    );
    assert!(
        body_bounds
            .windows(2)
            .all(|window| window[1].0 > window[0].1),
        "expected body bounds to be non-overlapping under dense spacing"
    );

    let mut wick_bounds: Vec<(i64, i64)> = series
        .lines
        .iter()
        .map(|line| {
            let width = line.stroke_width.round().max(1.0) as i64;
            let left = (line.x1 - (line.stroke_width - 1.0) * 0.5).round() as i64;
            let right = left + width - 1;
            (left, right)
        })
        .collect();
    wick_bounds.sort_by_key(|(left, _)| *left);
    assert_eq!(
        wick_bounds.len(),
        candle_count,
        "expected one wick per candle"
    );
    assert!(
        wick_bounds
            .windows(2)
            .all(|window| window[1].0 > window[0].1),
        "expected wick bounds to be non-overlapping under dense spacing"
    );
}
