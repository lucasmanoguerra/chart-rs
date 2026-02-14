use crate::render::Renderer;

use super::axis_label_format::map_price_to_display_value;
use super::layout_helpers::estimate_label_text_width_px;
use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn estimate_last_price_axis_label_required_width(
        &self,
        style: RenderStyle,
        visible_start: f64,
        visible_end: f64,
        fallback_display_base_price: f64,
        display_tick_step_abs: f64,
        display_suffix: &str,
    ) -> f64 {
        if !style.show_last_price_label {
            return 0.0;
        }

        let Some((last_price, _previous_price)) = self.resolve_latest_and_previous_price_values(
            style.last_price_source_mode,
            visible_start,
            visible_end,
        ) else {
            return 0.0;
        };

        let display_price = map_price_to_display_value(
            last_price,
            self.core.behavior.price_axis_label_config.display_mode,
            fallback_display_base_price,
        );
        let text =
            self.format_price_axis_label(display_price, display_tick_step_abs, display_suffix);
        let text_width = estimate_label_text_width_px(&text, style.last_price_label_font_size_px);
        let padding_right = if style.show_last_price_label_box {
            (2.0 * style.last_price_label_box_padding_x_px)
                .max(style.last_price_label_padding_right_px)
        } else {
            style.last_price_label_padding_right_px
        };

        text_width + padding_right + 2.0
    }
}
