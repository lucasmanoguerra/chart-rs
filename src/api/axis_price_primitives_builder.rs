use crate::render::{CanvasLayerKind, Renderer, TextHAlign, TextPrimitive};

use super::axis_label_format::map_price_to_display_value;
use super::axis_render_frame_builder::AxisPrimitiveSink;
use super::{ChartEngine, RenderStyle};

#[derive(Debug, Clone, Copy)]
pub(super) struct AxisPricePrimitivesContext {
    pub plot_right: f64,
    pub plot_bottom: f64,
    pub price_axis_label_anchor_x: f64,
    pub price_axis_tick_mark_end_x: f64,
    pub fallback_display_base_price: f64,
    pub display_tick_step_abs: f64,
    pub display_suffix: &'static str,
    pub style: RenderStyle,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_price_axis_tick_primitives(
        &self,
        sink: &mut AxisPrimitiveSink<'_>,
        ticks: Vec<(f64, f64)>,
        ctx: AxisPricePrimitivesContext,
    ) {
        let plot_right = ctx.plot_right;
        let plot_bottom = ctx.plot_bottom;
        let price_axis_label_anchor_x = ctx.price_axis_label_anchor_x;
        let price_axis_tick_mark_end_x = ctx.price_axis_tick_mark_end_x;
        let fallback_display_base_price = ctx.fallback_display_base_price;
        let display_tick_step_abs = ctx.display_tick_step_abs;
        let display_suffix = ctx.display_suffix;
        let style = ctx.style;
        let price_label_color = style.axis_label_color;

        for (price, py) in ticks {
            let display_price = map_price_to_display_value(
                price,
                self.core.behavior.price_axis_label_config.display_mode,
                fallback_display_base_price,
            );
            let text =
                self.format_price_axis_label(display_price, display_tick_step_abs, display_suffix);
            if style.show_price_axis_labels {
                let price_label_y = (py - style.price_axis_label_offset_y_px).clamp(
                    0.0,
                    (plot_bottom - style.price_axis_label_font_size_px).max(0.0),
                );
                sink.push_text(
                    CanvasLayerKind::Axis,
                    TextPrimitive::new(
                        text,
                        price_axis_label_anchor_x,
                        price_label_y,
                        style.price_axis_label_font_size_px,
                        price_label_color,
                        TextHAlign::Right,
                    ),
                );
            }
            if style.show_price_axis_grid_lines {
                sink.push_line(
                    CanvasLayerKind::Grid,
                    crate::render::LinePrimitive::new(
                        0.0,
                        py,
                        plot_right,
                        py,
                        style.price_axis_grid_line_width,
                        style.price_axis_grid_line_color,
                    ),
                );
            }
            if style.show_price_axis_tick_marks {
                sink.push_line(
                    CanvasLayerKind::Axis,
                    crate::render::LinePrimitive::new(
                        plot_right,
                        py,
                        price_axis_tick_mark_end_x,
                        py,
                        style.price_axis_tick_mark_width,
                        style.price_axis_tick_mark_color,
                    ),
                );
            }
        }
    }
}
