use crate::error::ChartResult;
use crate::render::Renderer;

use super::validation::validate_render_style;
use super::{
    RenderStyle,
    engine_core::EngineCore,
    render_coordinator::RenderCoordinator,
    render_style_invalidation_resolver::{
        RenderStyleInvalidationDecision, resolve_render_style_invalidation,
    },
};

#[cfg(feature = "cairo-backend")]
use crate::render::CairoContextRenderer;

/// Main orchestration facade consumed by host applications.
///
/// `ChartEngine` coordinates time/price scales, interaction state,
/// data/candle collections, and renderer calls.
pub struct ChartEngine<R: Renderer> {
    pub(super) renderer: R,
    pub(super) core: EngineCore,
}

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn render_style(&self) -> RenderStyle {
        self.core.presentation.render_style
    }

    pub fn set_render_style(&mut self, style: RenderStyle) -> ChartResult<()> {
        if self.core.presentation.render_style == style {
            return Ok(());
        }
        validate_render_style(style)?;
        let previous = self.core.presentation.render_style;
        self.core.presentation.render_style = style;
        match resolve_render_style_invalidation(previous, style) {
            RenderStyleInvalidationDecision::None => {}
            RenderStyleInvalidationDecision::Full => {
                self.invalidate_full();
            }
            RenderStyleInvalidationDecision::Light(topics) => {
                self.invalidate_with_detail(super::InvalidationLevel::Light, topics, None);
            }
        }
        Ok(())
    }

    pub fn render(&mut self) -> ChartResult<()> {
        RenderCoordinator::render(self)
    }

    /// Renders the frame into an external cairo context.
    ///
    /// This path is used by GTK draw callbacks while keeping the renderer
    /// implementation decoupled from GTK-specific APIs.
    #[cfg(feature = "cairo-backend")]
    pub fn render_on_cairo_context(&mut self, context: &cairo::Context) -> ChartResult<()>
    where
        R: CairoContextRenderer,
    {
        RenderCoordinator::render_on_cairo_context(self, context)
    }

    #[must_use]
    pub fn into_renderer(self) -> R {
        self.renderer
    }
}
