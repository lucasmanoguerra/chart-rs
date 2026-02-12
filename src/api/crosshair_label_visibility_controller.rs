use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, CrosshairAxisLabelVisibilityBehavior};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn crosshair_axis_label_visibility_behavior(&self) -> CrosshairAxisLabelVisibilityBehavior {
        let style = self.render_style();
        CrosshairAxisLabelVisibilityBehavior {
            show_time_label: style.show_crosshair_time_label,
            show_price_label: style.show_crosshair_price_label,
            show_time_label_box: style.show_crosshair_time_label_box,
            show_price_label_box: style.show_crosshair_price_label_box,
            show_time_label_box_border: style.show_crosshair_time_label_box_border,
            show_price_label_box_border: style.show_crosshair_price_label_box_border,
        }
    }

    pub fn set_crosshair_axis_label_visibility_behavior(
        &mut self,
        behavior: CrosshairAxisLabelVisibilityBehavior,
    ) -> ChartResult<()> {
        let mut style = self.render_style();
        style.show_crosshair_time_label = behavior.show_time_label;
        style.show_crosshair_price_label = behavior.show_price_label;
        style.show_crosshair_time_label_box = behavior.show_time_label_box;
        style.show_crosshair_price_label_box = behavior.show_price_label_box;
        style.show_crosshair_time_label_box_border = behavior.show_time_label_box_border;
        style.show_crosshair_price_label_box_border = behavior.show_price_label_box_border;
        self.set_render_style(style)
    }
}
