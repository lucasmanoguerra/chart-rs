use crate::error::ChartResult;
use crate::extensions::PluginEvent;
use crate::render::Renderer;

use super::ChartEngine;
#[cfg(feature = "cairo-backend")]
use super::pane_render_executor::PaneRenderExecutor;
#[cfg(feature = "cairo-backend")]
use super::render_partial_scheduler::PartialCairoRenderPlan;

#[cfg(feature = "cairo-backend")]
use crate::render::CairoContextRenderer;

pub(super) struct RenderCoordinator;

impl RenderCoordinator {
    pub(super) fn render<R: Renderer>(engine: &mut ChartEngine<R>) -> ChartResult<()> {
        let frame = engine.build_render_frame()?;
        engine.renderer.render(&frame)?;
        Self::finalize_render(engine);
        Ok(())
    }

    #[cfg(feature = "cairo-backend")]
    pub(super) fn render_on_cairo_context<R: Renderer + CairoContextRenderer>(
        engine: &mut ChartEngine<R>,
        context: &cairo::Context,
    ) -> ChartResult<()> {
        let pending_invalidation = engine.pending_invalidation();
        let api_pane_targets = engine.pending_invalidation_pane_targets();
        let lwc_pending_invalidation = engine.lwc_pending_invalidation();
        let lwc_pane_ids = engine
            .core
            .lwc_model
            .panes()
            .iter()
            .map(|pane| pane.id())
            .collect::<Vec<_>>();
        if engine.panes().len() > 1 {
            let layered = engine.build_layered_render_frame()?;
            if let Some(plan) = PartialCairoRenderPlan::build_from_masks(
                pending_invalidation,
                &api_pane_targets,
                lwc_pending_invalidation,
                &lwc_pane_ids,
                &layered,
            ) {
                let tasks = PaneRenderExecutor::collect_partial_tasks(engine, &layered, &plan);
                for task in tasks {
                    engine.renderer.render_on_cairo_context_partial(
                        context,
                        &task.frame,
                        task.clip_rect,
                        task.clear_region,
                    )?;
                }

                Self::finalize_render(engine);
                return Ok(());
            }
        }

        let frame = engine.build_render_frame()?;
        engine.renderer.render_on_cairo_context(context, &frame)?;
        Self::finalize_render(engine);
        Ok(())
    }

    fn finalize_render<R: Renderer>(engine: &mut ChartEngine<R>) {
        engine.clear_pending_invalidation();
        engine.emit_plugin_event(PluginEvent::Rendered);
    }
}
