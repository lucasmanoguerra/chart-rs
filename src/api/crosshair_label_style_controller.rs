use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, CrosshairAxisLabelStyleBehavior};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn crosshair_axis_label_style_behavior(&self) -> CrosshairAxisLabelStyleBehavior {
        let style = self.render_style();
        CrosshairAxisLabelStyleBehavior {
            time_label_color: style.crosshair_time_label_color,
            price_label_color: style.crosshair_price_label_color,
            time_label_font_size_px: style.crosshair_time_label_font_size_px,
            price_label_font_size_px: style.crosshair_price_label_font_size_px,
            time_label_offset_y_px: style.crosshair_time_label_offset_y_px,
            price_label_offset_y_px: style.crosshair_price_label_offset_y_px,
            time_label_padding_x_px: style.crosshair_time_label_padding_x_px,
            price_label_padding_right_px: style.crosshair_price_label_padding_right_px,
        }
    }

    pub fn set_crosshair_axis_label_style_behavior(
        &mut self,
        behavior: CrosshairAxisLabelStyleBehavior,
    ) -> ChartResult<()> {
        let mut style = self.render_style();
        style.crosshair_time_label_color = behavior.time_label_color;
        style.crosshair_price_label_color = behavior.price_label_color;
        style.crosshair_time_label_font_size_px = behavior.time_label_font_size_px;
        style.crosshair_price_label_font_size_px = behavior.price_label_font_size_px;
        style.crosshair_time_label_offset_y_px = behavior.time_label_offset_y_px;
        style.crosshair_price_label_offset_y_px = behavior.price_label_offset_y_px;
        style.crosshair_time_label_padding_x_px = behavior.time_label_padding_x_px;
        style.crosshair_price_label_padding_right_px = behavior.price_label_padding_right_px;
        self.set_render_style(style)
    }
}
