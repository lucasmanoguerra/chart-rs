#[cfg(feature = "cairo-backend")]
use crate::error::ChartResult;
#[cfg(feature = "cairo-backend")]
use crate::render::{LayeredRenderFrame, Renderer};

#[cfg(feature = "cairo-backend")]
use super::ChartEngine;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_partial_input_resolver::collect_cairo_partial_render_inputs;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_partial_plan_resolver::resolve_cairo_partial_render_plan;
#[cfg(feature = "cairo-backend")]
use super::render_partial_scheduler::PartialCairoRenderPlan;

#[cfg(feature = "cairo-backend")]
pub(super) enum CairoRenderExecutionPath {
    Full,
    Partial {
        layered: LayeredRenderFrame,
        plan: PartialCairoRenderPlan,
    },
}

#[cfg(feature = "cairo-backend")]
impl CairoRenderExecutionPath {
    pub(super) fn resolve<R: Renderer>(engine: &ChartEngine<R>) -> ChartResult<Self> {
        if engine.panes().len() <= 1 {
            return Ok(Self::Full);
        }

        let inputs = collect_cairo_partial_render_inputs(engine);
        let Some((layered, plan)) = resolve_cairo_partial_render_plan(engine, &inputs)? else {
            return Ok(Self::Full);
        };
        Ok(Self::Partial { layered, plan })
    }
}

#[cfg(all(test, feature = "cairo-backend"))]
mod tests {
    use super::CairoRenderExecutionPath;
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
    fn resolve_returns_full_for_single_pane() {
        let engine = build_engine();
        let path = CairoRenderExecutionPath::resolve(&engine).expect("resolve path");
        assert!(matches!(path, CairoRenderExecutionPath::Full));
    }

    #[test]
    fn resolve_returns_partial_for_multi_pane_cursor_invalidation() {
        let mut engine = build_engine();
        let _ = engine.create_pane(1.0).expect("create pane");
        engine.clear_pending_invalidation();
        engine.invalidate_cursor();

        let path = CairoRenderExecutionPath::resolve(&engine).expect("resolve path");
        assert!(matches!(path, CairoRenderExecutionPath::Partial { .. }));
    }
}
