use crate::error::ChartResult;
use crate::render::Renderer;

use super::layout_helpers::AxisLayout;
use super::{ChartEngine, RenderStyle};

#[derive(Debug, Clone, Copy)]
pub(super) struct ResolvedRenderAxisLayout {
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub visible_span_abs: f64,
    pub axis_layout: AxisLayout,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_render_axis_layout(
        &self,
        style: RenderStyle,
        visible_start: f64,
        visible_end: f64,
    ) -> ChartResult<ResolvedRenderAxisLayout> {
        let viewport_width = f64::from(self.core.model.viewport.width);
        let viewport_height = f64::from(self.core.model.viewport.height);
        let visible_span_abs = (visible_end - visible_start).abs();

        let axis_layout = self.resolve_adaptive_axis_layout(
            style,
            viewport_width,
            viewport_height,
            visible_start,
            visible_end,
        )?;

        Ok(ResolvedRenderAxisLayout {
            viewport_width,
            viewport_height,
            visible_span_abs,
            axis_layout,
        })
    }
}
