use crate::render::{CanvasLayerKind, Renderer, TextHAlign, TextPrimitive};

use super::ChartEngine;
use super::axis_label_format::map_price_to_display_value;
use super::axis_render_frame_builder::AxisPrimitiveSink;
use super::last_price_axis_label_layout_builder::{
    LastPriceAxisLabelLayoutContext, build_last_price_axis_label_layout,
};
use super::last_price_axis_scene_builder::{LastPriceAxisSceneContext, LastPriceMarker};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_last_price_axis_label_primitives(
        &self,
        sink: &mut AxisPrimitiveSink<'_>,
        marker: LastPriceMarker,
        ctx: LastPriceAxisSceneContext,
    ) {
        let plot_right = ctx.plot_right;
        let plot_bottom = ctx.plot_bottom;
        let viewport_width = ctx.viewport_width;
        let last_price_label_anchor_x = ctx.last_price_label_anchor_x;
        let fallback_display_base_price = ctx.fallback_display_base_price;
        let display_tick_step_abs = ctx.display_tick_step_abs;
        let display_suffix = ctx.display_suffix;
        let style = ctx.style;

        if !style.show_last_price_label {
            return;
        }

        let display_price = map_price_to_display_value(
            marker.last_price,
            self.core.behavior.price_axis_label_config.display_mode,
            fallback_display_base_price,
        );
        let text =
            self.format_price_axis_label(display_price, display_tick_step_abs, display_suffix);
        let box_fill_color =
            self.resolve_last_price_label_box_fill_color(marker.marker_label_color);
        let label_text_color =
            self.resolve_last_price_label_box_text_color(box_fill_color, marker.marker_label_color);
        let default_text_anchor_x = last_price_label_anchor_x;
        let layout = build_last_price_axis_label_layout(LastPriceAxisLabelLayoutContext {
            marker_py: marker.py,
            text: &text,
            plot_right,
            plot_bottom,
            viewport_width,
            default_text_anchor_x,
            box_fill_color,
            style,
        });
        if let Some(rect) = layout.box_rect {
            sink.push_rect(CanvasLayerKind::Axis, rect);
        }
        sink.push_text(
            CanvasLayerKind::Axis,
            TextPrimitive::new(
                text,
                layout.text_anchor_x,
                layout.text_y,
                style.last_price_label_font_size_px,
                label_text_color,
                TextHAlign::Right,
            ),
        );
    }
}
