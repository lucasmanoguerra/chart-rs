use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, CrosshairAxisLabelBoxStyleBehavior};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn crosshair_axis_label_box_style_behavior(&self) -> CrosshairAxisLabelBoxStyleBehavior {
        let style = self.render_style();
        CrosshairAxisLabelBoxStyleBehavior {
            box_color: style.crosshair_label_box_color,
            time_box_color: style.crosshair_time_label_box_color,
            price_box_color: style.crosshair_price_label_box_color,
            box_border_color: style.crosshair_label_box_border_color,
            time_box_border_color: style.crosshair_time_label_box_border_color,
            price_box_border_color: style.crosshair_price_label_box_border_color,
            box_border_width_px: style.crosshair_label_box_border_width_px,
            time_box_border_width_px: style.crosshair_time_label_box_border_width_px,
            price_box_border_width_px: style.crosshair_price_label_box_border_width_px,
            box_corner_radius_px: style.crosshair_label_box_corner_radius_px,
            time_box_corner_radius_px: style.crosshair_time_label_box_corner_radius_px,
            price_box_corner_radius_px: style.crosshair_price_label_box_corner_radius_px,
        }
    }

    pub fn set_crosshair_axis_label_box_style_behavior(
        &mut self,
        behavior: CrosshairAxisLabelBoxStyleBehavior,
    ) -> ChartResult<()> {
        let mut style = self.render_style();
        style.crosshair_label_box_color = behavior.box_color;
        style.crosshair_time_label_box_color = behavior.time_box_color;
        style.crosshair_price_label_box_color = behavior.price_box_color;
        style.crosshair_label_box_border_color = behavior.box_border_color;
        style.crosshair_time_label_box_border_color = behavior.time_box_border_color;
        style.crosshair_price_label_box_border_color = behavior.price_box_border_color;
        style.crosshair_label_box_border_width_px = behavior.box_border_width_px;
        style.crosshair_time_label_box_border_width_px = behavior.time_box_border_width_px;
        style.crosshair_price_label_box_border_width_px = behavior.price_box_border_width_px;
        style.crosshair_label_box_corner_radius_px = behavior.box_corner_radius_px;
        style.crosshair_time_label_box_corner_radius_px = behavior.time_box_corner_radius_px;
        style.crosshair_price_label_box_corner_radius_px = behavior.price_box_corner_radius_px;
        self.set_render_style(style)
    }
}
