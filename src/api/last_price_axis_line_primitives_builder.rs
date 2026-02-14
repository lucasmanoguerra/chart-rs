use crate::render::{CanvasLayerKind, Renderer};

use super::ChartEngine;
use super::axis_render_frame_builder::AxisPrimitiveSink;
use super::last_price_axis_scene_builder::{LastPriceAxisSceneContext, LastPriceMarker};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_last_price_axis_line_primitive(
        &self,
        sink: &mut AxisPrimitiveSink<'_>,
        marker: LastPriceMarker,
        ctx: LastPriceAxisSceneContext,
    ) {
        let plot_right = ctx.plot_right;
        let style = ctx.style;

        if style.show_last_price_line {
            sink.push_line(
                CanvasLayerKind::Overlay,
                crate::render::LinePrimitive::new(
                    0.0,
                    marker.py,
                    plot_right,
                    marker.py,
                    style.last_price_line_width,
                    marker.marker_line_color,
                ),
            );
        }
    }
}
