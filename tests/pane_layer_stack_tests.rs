use chart_rs::api::{ChartEngine, ChartEngineConfig, InvalidationLevel};
use chart_rs::core::{PaneId, Viewport};
use chart_rs::render::{CanvasLayerKind, NullRenderer};

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn engine_initializes_with_single_main_pane_and_canonical_stack() {
    let engine = build_engine();
    let panes = engine.panes();
    assert_eq!(panes.len(), 1);
    assert!(panes[0].is_main);
    assert_eq!(engine.main_pane_id(), PaneId::new(0));

    let stack = engine
        .pane_layer_stack(engine.main_pane_id())
        .expect("main pane stack");
    assert_eq!(
        stack.layers,
        vec![
            CanvasLayerKind::Background,
            CanvasLayerKind::Grid,
            CanvasLayerKind::Series,
            CanvasLayerKind::Overlay,
            CanvasLayerKind::Crosshair,
            CanvasLayerKind::Axis,
        ]
    );
}

#[test]
fn create_and_remove_auxiliary_pane_updates_collection() {
    let mut engine = build_engine();
    engine.clear_pending_invalidation();

    let pane_id = engine.create_pane(2.0).expect("create pane");
    assert_ne!(pane_id, engine.main_pane_id());
    assert_eq!(engine.panes().len(), 2);
    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::Full);

    engine.clear_pending_invalidation();
    let removed = engine.remove_pane(pane_id).expect("remove pane");
    assert!(removed);
    assert_eq!(engine.panes().len(), 1);
    assert_eq!(engine.pending_invalidation_level(), InvalidationLevel::Full);
}

#[test]
fn removing_assigned_pane_rebinds_series_to_main_pane() {
    let mut engine = build_engine();
    let aux = engine.create_pane(1.0).expect("create pane");

    engine.set_points_pane(aux).expect("assign points pane");
    engine.set_candles_pane(aux).expect("assign candles pane");
    assert_eq!(engine.points_pane_id(), aux);
    assert_eq!(engine.candles_pane_id(), aux);

    let removed = engine.remove_pane(aux).expect("remove pane");
    assert!(removed);
    assert_eq!(engine.points_pane_id(), engine.main_pane_id());
    assert_eq!(engine.candles_pane_id(), engine.main_pane_id());
}

#[test]
fn pane_plot_regions_for_current_viewport_cover_plot_area() {
    let mut engine = build_engine();
    let _pane_a = engine.create_pane(1.0).expect("create pane A");
    let _pane_b = engine.create_pane(2.0).expect("create pane B");

    let regions = engine.pane_plot_regions_for_current_viewport();
    assert_eq!(regions.len(), 3);
    assert!((regions[0].plot_top - 0.0).abs() <= 1e-9);
    for pair in regions.windows(2) {
        assert!((pair[0].plot_bottom - pair[1].plot_top).abs() <= 1e-9);
    }
    let final_bottom = regions
        .last()
        .map(|region| region.plot_bottom)
        .expect("at least one region");
    assert!(final_bottom > 0.0);
}

#[test]
fn assigning_series_to_unknown_pane_fails() {
    let mut engine = build_engine();
    let unknown = PaneId::new(999);
    let points_err = engine
        .set_points_pane(unknown)
        .expect_err("unknown points pane must fail");
    assert!(
        points_err
            .to_string()
            .contains("points pane does not exist")
    );

    let candles_err = engine
        .set_candles_pane(unknown)
        .expect_err("unknown candles pane must fail");
    assert!(
        candles_err
            .to_string()
            .contains("candles pane does not exist")
    );
}

#[test]
fn main_pane_cannot_be_removed() {
    let mut engine = build_engine();
    let main = engine.main_pane_id();
    let error = engine
        .remove_pane(main)
        .expect_err("main pane removal must fail");
    assert!(error.to_string().contains("cannot remove main pane"));
}
