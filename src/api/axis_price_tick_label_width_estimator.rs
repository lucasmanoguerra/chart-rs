use crate::render::Renderer;

use super::axis_label_format::map_price_to_display_value;
use super::layout_helpers::estimate_label_text_width_px;
use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn estimate_price_axis_tick_labels_required_width(
        &self,
        style: RenderStyle,
        selected_price_ticks: &[(f64, f64)],
        fallback_display_base_price: f64,
        display_tick_step_abs: f64,
        display_suffix: &str,
    ) -> f64 {
        if !style.show_price_axis_labels {
            return 0.0;
        }

        let mut required_width: f64 = 0.0;
        for (price, _) in selected_price_ticks.iter().copied() {
            let display_price = map_price_to_display_value(
                price,
                self.core.behavior.price_axis_label_config.display_mode,
                fallback_display_base_price,
            );
            let text =
                self.format_price_axis_label(display_price, display_tick_step_abs, display_suffix);
            let text_width =
                estimate_label_text_width_px(&text, style.price_axis_label_font_size_px);
            required_width =
                required_width.max(text_width + style.price_axis_label_padding_right_px + 2.0);
        }

        required_width
    }
}
