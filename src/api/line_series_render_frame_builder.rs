use crate::core::{PaneId, PriceScale, points_in_time_window, project_line_segments};
use crate::error::ChartResult;
use crate::render::{
    CanvasLayerKind, Color, LayeredRenderFrame, LinePrimitive, RenderFrame, Renderer,
};

use super::ChartEngine;

#[derive(Debug, Clone, Copy)]
pub(super) struct LineSeriesRenderContext {
    pub pane_id: PaneId,
    pub price_scale: PriceScale,
    pub visible_start: f64,
    pub visible_end: f64,
    pub line_color: Color,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_line_series_primitives(
        &self,
        frame: &mut RenderFrame,
        layered: &mut LayeredRenderFrame,
        ctx: LineSeriesRenderContext,
    ) -> ChartResult<()> {
        let pane_id = ctx.pane_id;
        let price_scale = ctx.price_scale;
        let visible_start = ctx.visible_start;
        let visible_end = ctx.visible_end;
        let line_color = ctx.line_color;

        let visible_points =
            points_in_time_window(&self.core.model.points, visible_start, visible_end);
        let segments = project_line_segments(
            &visible_points,
            self.core.model.time_scale,
            price_scale,
            self.core.model.viewport,
        )?;

        for segment in segments {
            let line = LinePrimitive::new(
                segment.x1, segment.y1, segment.x2, segment.y2, 1.5, line_color,
            );
            frame.lines.push(line);
            layered.push_line(pane_id, CanvasLayerKind::Series, line);
        }

        Ok(())
    }
}
