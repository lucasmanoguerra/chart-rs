use crate::render::Renderer;

use super::ChartEngine;
use super::axis_label_format::{map_price_step_to_display_value, price_display_mode_suffix};
use super::axis_render_frame_builder::AxisPriceDisplayContext;

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_price_axis_display_context(
        &self,
        raw_tick_step_abs: f64,
    ) -> AxisPriceDisplayContext {
        let fallback_display_base_price = self.resolve_price_display_base_price();
        let display_suffix =
            price_display_mode_suffix(self.core.behavior.price_axis_label_config.display_mode);
        let display_tick_step_abs = map_price_step_to_display_value(
            raw_tick_step_abs,
            self.core.behavior.price_axis_label_config.display_mode,
            fallback_display_base_price,
        )
        .abs();

        AxisPriceDisplayContext {
            fallback_display_base_price,
            display_tick_step_abs,
            display_suffix,
        }
    }
}
