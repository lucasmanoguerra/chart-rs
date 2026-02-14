use crate::error::ChartResult;
use crate::render::Renderer;

use super::ChartEngine;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_coordinator::render_on_cairo_context as render_cairo_path;
use super::render_cycle_finalizer::finalize_render_cycle;
use super::render_full_pass_executor::render_full_pass;

#[cfg(feature = "cairo-backend")]
use crate::render::CairoContextRenderer;

pub(super) struct RenderCoordinator;

impl RenderCoordinator {
    pub(super) fn render<R: Renderer>(engine: &mut ChartEngine<R>) -> ChartResult<()> {
        render_full_pass(engine)?;
        finalize_render_cycle(engine);
        Ok(())
    }

    #[cfg(feature = "cairo-backend")]
    pub(super) fn render_on_cairo_context<R: Renderer + CairoContextRenderer>(
        engine: &mut ChartEngine<R>,
        context: &cairo::Context,
    ) -> ChartResult<()> {
        render_cairo_path(engine, context)
    }
}
