use super::axis_render_frame_builder::AxisPrimitiveSink;
use super::{ChartEngine, RenderStyle};
use crate::render::{Color, Renderer};

#[derive(Debug, Clone, Copy)]
pub(super) struct LastPriceMarker {
    pub last_price: f64,
    pub py: f64,
    pub marker_line_color: Color,
    pub marker_label_color: Color,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LastPriceAxisSceneContext {
    pub plot_right: f64,
    pub plot_bottom: f64,
    pub viewport_width: f64,
    pub last_price_label_anchor_x: f64,
    pub fallback_display_base_price: f64,
    pub display_tick_step_abs: f64,
    pub display_suffix: &'static str,
    pub style: RenderStyle,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_last_price_axis_primitives(
        &self,
        sink: &mut AxisPrimitiveSink<'_>,
        marker: Option<LastPriceMarker>,
        ctx: LastPriceAxisSceneContext,
    ) {
        let Some(marker) = marker else {
            return;
        };

        self.append_last_price_axis_line_primitive(sink, marker, ctx);
        self.append_last_price_axis_label_primitives(sink, marker, ctx);
    }
}
