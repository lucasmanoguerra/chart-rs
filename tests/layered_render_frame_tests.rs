use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairMode};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::{CanvasLayerKind, NullRenderer};

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 480), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(10.0, 10.0),
        DataPoint::new(30.0, 20.0),
        DataPoint::new(70.0, 18.0),
    ]);
    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine.pointer_move(200.0, 120.0);
    engine
}

fn build_candles_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 480), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_candles(vec![
        OhlcBar::new(10.0, 10.0, 13.0, 9.0, 12.0).expect("candle 1"),
        OhlcBar::new(30.0, 12.0, 14.0, 11.0, 11.5).expect("candle 2"),
        OhlcBar::new(70.0, 11.5, 15.0, 10.5, 14.0).expect("candle 3"),
    ]);
    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine.pointer_move(240.0, 160.0);
    engine
}

#[test]
fn layered_render_frame_flatten_preserves_primitive_counts() {
    let engine = build_engine();
    let legacy = engine.build_render_frame().expect("legacy frame");
    let layered = engine
        .build_layered_render_frame()
        .expect("layered render frame");

    let flattened = layered.flatten();
    assert_eq!(flattened.viewport, legacy.viewport);
    assert_eq!(flattened.lines.len(), legacy.lines.len());
    assert_eq!(flattened.rects.len(), legacy.rects.len());
    assert_eq!(flattened.texts.len(), legacy.texts.len());
}

#[test]
fn layered_render_frame_respects_canonical_layer_order() {
    let engine = build_engine();
    let layered = engine
        .build_layered_render_frame()
        .expect("layered render frame");
    assert_eq!(layered.panes.len(), 1);

    let layers = &layered.panes[0].layers;
    assert_eq!(layers[0].kind, CanvasLayerKind::Background);
    assert_eq!(layers[1].kind, CanvasLayerKind::Grid);
    assert_eq!(layers[2].kind, CanvasLayerKind::Series);
    assert_eq!(layers[3].kind, CanvasLayerKind::Overlay);
    assert_eq!(layers[4].kind, CanvasLayerKind::Crosshair);
    assert_eq!(layers[5].kind, CanvasLayerKind::Axis);
}

#[test]
fn layered_render_frame_includes_all_panes_even_when_secondary_is_empty() {
    let mut engine = build_engine();
    let _aux = engine.create_pane(1.0).expect("create secondary pane");

    let layered = engine
        .build_layered_render_frame()
        .expect("layered render frame");
    assert_eq!(layered.panes.len(), 2);
    let total_height: f64 = layered
        .panes
        .iter()
        .map(|pane| pane.plot_bottom - pane.plot_top)
        .sum();
    let plot_bottom = layered
        .panes
        .iter()
        .map(|pane| pane.plot_bottom)
        .fold(0.0_f64, f64::max);
    assert!((total_height - plot_bottom).abs() <= 1e-9);
    assert!(
        layered.panes[1]
            .layers
            .iter()
            .all(|layer| layer.lines.is_empty()
                && layer.rects.is_empty()
                && layer.texts.is_empty())
    );
}

#[test]
fn build_render_frame_for_pane_returns_scoped_primitives() {
    let mut engine = build_engine();
    let aux = engine.create_pane(1.0).expect("create secondary pane");

    let main_frame = engine
        .build_render_frame_for_pane(engine.main_pane_id())
        .expect("main pane frame")
        .expect("main pane exists");
    assert!(!main_frame.lines.is_empty());

    let aux_frame = engine
        .build_render_frame_for_pane(aux)
        .expect("aux pane frame")
        .expect("aux pane exists");
    assert!(aux_frame.lines.is_empty());
    assert!(aux_frame.rects.is_empty());
    assert!(aux_frame.texts.is_empty());
}

#[test]
fn layered_plot_layers_are_remapped_into_main_pane_region() {
    let mut engine = build_engine();
    let _aux = engine.create_pane(1.0).expect("create secondary pane");
    let layered = engine
        .build_layered_render_frame()
        .expect("layered render frame");
    let main = layered
        .panes
        .iter()
        .find(|pane| pane.pane_id == engine.main_pane_id())
        .expect("main pane");
    let series_layer = main
        .layers
        .iter()
        .find(|layer| layer.kind == CanvasLayerKind::Series)
        .expect("series layer");
    assert!(!series_layer.lines.is_empty());
    for line in &series_layer.lines {
        assert!(line.y1 >= main.plot_top - 1e-9);
        assert!(line.y1 <= main.plot_bottom + 1e-9);
        assert!(line.y2 >= main.plot_top - 1e-9);
        assert!(line.y2 <= main.plot_bottom + 1e-9);
    }
}

#[test]
fn points_series_can_be_routed_to_aux_pane_with_local_remap() {
    let mut engine = build_engine();
    let aux = engine.create_pane(1.0).expect("create secondary pane");
    engine
        .set_points_pane(aux)
        .expect("assign points pane to aux");

    let layered = engine
        .build_layered_render_frame()
        .expect("layered render frame");
    let main = layered
        .panes
        .iter()
        .find(|pane| pane.pane_id == engine.main_pane_id())
        .expect("main pane");
    let aux_pane = layered
        .panes
        .iter()
        .find(|pane| pane.pane_id == aux)
        .expect("aux pane");

    let main_series = main
        .layers
        .iter()
        .find(|layer| layer.kind == CanvasLayerKind::Series)
        .expect("main series layer");
    let aux_series = aux_pane
        .layers
        .iter()
        .find(|layer| layer.kind == CanvasLayerKind::Series)
        .expect("aux series layer");

    assert!(main_series.lines.is_empty());
    assert!(!aux_series.lines.is_empty());
    for line in &aux_series.lines {
        assert!(line.y1 >= aux_pane.plot_top - 1e-9);
        assert!(line.y1 <= aux_pane.plot_bottom + 1e-9);
        assert!(line.y2 >= aux_pane.plot_top - 1e-9);
        assert!(line.y2 <= aux_pane.plot_bottom + 1e-9);
    }
}

#[test]
fn candlestick_series_can_be_routed_to_aux_pane_with_local_remap() {
    let mut engine = build_candles_engine();
    let aux = engine.create_pane(1.0).expect("create secondary pane");
    engine
        .set_candles_pane(aux)
        .expect("assign candles pane to aux");

    let layered = engine
        .build_layered_render_frame()
        .expect("layered render frame");
    let main = layered
        .panes
        .iter()
        .find(|pane| pane.pane_id == engine.main_pane_id())
        .expect("main pane");
    let aux_pane = layered
        .panes
        .iter()
        .find(|pane| pane.pane_id == aux)
        .expect("aux pane");

    let main_series = main
        .layers
        .iter()
        .find(|layer| layer.kind == CanvasLayerKind::Series)
        .expect("main series layer");
    let aux_series = aux_pane
        .layers
        .iter()
        .find(|layer| layer.kind == CanvasLayerKind::Series)
        .expect("aux series layer");

    assert!(main_series.rects.is_empty());
    assert!(main_series.lines.is_empty());
    assert!(
        !aux_series.lines.is_empty(),
        "expected wick lines in aux pane series layer"
    );
    assert!(
        !aux_series.rects.is_empty(),
        "expected candle bodies in aux pane series layer"
    );
    for line in &aux_series.lines {
        assert!(line.y1 >= aux_pane.plot_top - 1e-9);
        assert!(line.y1 <= aux_pane.plot_bottom + 1e-9);
        assert!(line.y2 >= aux_pane.plot_top - 1e-9);
        assert!(line.y2 <= aux_pane.plot_bottom + 1e-9);
    }
}
