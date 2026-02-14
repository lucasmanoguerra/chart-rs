use crate::error::ChartResult;
use crate::extensions::PluginEvent;
use crate::render::Renderer;

use super::ChartEngine;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_execution_path_resolver::CairoRenderExecutionPath;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_partial_pass_executor::render_partial_on_cairo_context;

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
        match CairoRenderExecutionPath::resolve(engine)? {
            CairoRenderExecutionPath::Partial { layered, plan } => {
                render_partial_on_cairo_context(engine, context, &layered, &plan)?;
                Self::finalize_render(engine);
                Ok(())
            }
            CairoRenderExecutionPath::Full => {
                let frame = engine.build_render_frame()?;
                engine.renderer.render_on_cairo_context(context, &frame)?;
                Self::finalize_render(engine);
                Ok(())
            }
        }
    }

    fn finalize_render<R: Renderer>(engine: &mut ChartEngine<R>) {
        engine.clear_pending_invalidation();
        engine.emit_plugin_event(PluginEvent::Rendered);
    }
}
