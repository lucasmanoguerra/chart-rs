#[cfg(feature = "cairo-backend")]
use crate::core::PaneId;
#[cfg(feature = "cairo-backend")]
use crate::render::Renderer;

#[cfg(feature = "cairo-backend")]
use super::{ChartEngine, InvalidationMask};

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Clone)]
pub(super) struct CairoPartialRenderInputs {
    pub(super) pending_invalidation: InvalidationMask,
    pub(super) api_pane_targets: Vec<PaneId>,
    pub(super) lwc_pending_invalidation: Option<crate::lwc::model::InvalidateMask>,
    pub(super) lwc_pane_ids: Vec<PaneId>,
}

#[cfg(feature = "cairo-backend")]
#[must_use]
pub(super) fn collect_cairo_partial_render_inputs<R: Renderer>(
    engine: &ChartEngine<R>,
) -> CairoPartialRenderInputs {
    let pending_invalidation = engine.pending_invalidation();
    let api_pane_targets = engine.pending_invalidation_pane_targets();
    let lwc_pending_invalidation = engine.lwc_pending_invalidation().cloned();
    let lwc_pane_ids = engine
        .core
        .lwc_model
        .panes()
        .iter()
        .map(|pane| pane.id())
        .collect::<Vec<_>>();

    CairoPartialRenderInputs {
        pending_invalidation,
        api_pane_targets,
        lwc_pending_invalidation,
        lwc_pane_ids,
    }
}

#[cfg(all(test, feature = "cairo-backend"))]
mod tests {
    use super::collect_cairo_partial_render_inputs;
    use crate::api::{ChartEngine, ChartEngineConfig};
    use crate::core::Viewport;
    use crate::render::NullRenderer;

    fn build_engine() -> ChartEngine<NullRenderer> {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        ChartEngine::new(renderer, config).expect("engine init")
    }

    #[test]
    fn collect_inputs_reflects_lwc_pane_ids() {
        let mut engine = build_engine();
        let main = engine.main_pane_id();
        let aux = engine.create_pane(1.0).expect("create pane");
        engine.clear_pending_invalidation();

        let inputs = collect_cairo_partial_render_inputs(&engine);
        assert_eq!(inputs.lwc_pane_ids, vec![main, aux]);
    }

    #[test]
    fn collect_inputs_clones_pending_lwc_mask() {
        let mut engine = build_engine();
        let _ = engine.create_pane(1.0).expect("create pane");
        engine.clear_pending_invalidation();
        engine.invalidate_cursor();

        let inputs = collect_cairo_partial_render_inputs(&engine);
        assert!(inputs.lwc_pending_invalidation.is_some());
    }
}
