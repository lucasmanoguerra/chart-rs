#[cfg(feature = "cairo-backend")]
use crate::error::ChartResult;
#[cfg(feature = "cairo-backend")]
use crate::render::{LayeredRenderFrame, Renderer};

#[cfg(feature = "cairo-backend")]
use super::ChartEngine;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_partial_input_resolver::CairoPartialRenderInputs;
#[cfg(feature = "cairo-backend")]
use super::render_partial_scheduler::PartialCairoRenderPlan;

#[cfg(feature = "cairo-backend")]
pub(super) fn resolve_cairo_partial_render_plan<R: Renderer>(
    engine: &ChartEngine<R>,
    inputs: &CairoPartialRenderInputs,
) -> ChartResult<Option<(LayeredRenderFrame, PartialCairoRenderPlan)>> {
    let layered = engine.build_layered_render_frame()?;
    let Some(plan) = PartialCairoRenderPlan::build_from_masks(
        inputs.pending_invalidation,
        &inputs.api_pane_targets,
        inputs.lwc_pending_invalidation.as_ref(),
        &inputs.lwc_pane_ids,
        &layered,
    ) else {
        return Ok(None);
    };

    Ok(Some((layered, plan)))
}

#[cfg(all(test, feature = "cairo-backend"))]
mod tests {
    use super::resolve_cairo_partial_render_plan;
    use crate::api::{ChartEngine, ChartEngineConfig, render_cairo_partial_input_resolver};
    use crate::core::Viewport;
    use crate::render::NullRenderer;

    fn build_engine() -> ChartEngine<NullRenderer> {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        ChartEngine::new(renderer, config).expect("engine init")
    }

    #[test]
    fn resolve_returns_none_without_partial_plan() {
        let engine = build_engine();
        let inputs =
            render_cairo_partial_input_resolver::collect_cairo_partial_render_inputs(&engine);

        let resolved = resolve_cairo_partial_render_plan(&engine, &inputs).expect("resolve");
        assert!(resolved.is_none());
    }

    #[test]
    fn resolve_returns_some_for_multi_pane_cursor_invalidation() {
        let mut engine = build_engine();
        let _ = engine.create_pane(1.0).expect("create pane");
        engine.clear_pending_invalidation();
        engine.invalidate_cursor();

        let inputs =
            render_cairo_partial_input_resolver::collect_cairo_partial_render_inputs(&engine);
        let resolved = resolve_cairo_partial_render_plan(&engine, &inputs).expect("resolve");
        assert!(resolved.is_some());
    }
}
