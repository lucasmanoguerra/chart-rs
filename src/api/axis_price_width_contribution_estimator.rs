use super::axis_price_width_contribution_accumulator::accumulate_price_axis_width_contribution;
use crate::render::Renderer;

use super::axis_price_width_display_input_resolver::resolve_price_axis_width_display_inputs;
use super::axis_price_width_selected_ticks_resolver::resolve_price_axis_width_selected_ticks;
use super::axis_price_width_tick_context_resolver::PriceAxisWidthTickContext;
use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn estimate_price_axis_width_contribution(
        &self,
        style: RenderStyle,
        visible_start: f64,
        visible_end: f64,
        width_tick_context: &PriceAxisWidthTickContext,
    ) -> f64 {
        let display_inputs = resolve_price_axis_width_display_inputs(width_tick_context);
        let selected_ticks = resolve_price_axis_width_selected_ticks(width_tick_context);
        let fallback_display_base_price = display_inputs.fallback_display_base_price;
        let display_tick_step_abs = display_inputs.display_tick_step_abs;
        let display_suffix = display_inputs.display_suffix;

        let mut required_width: f64 = 0.0;
        required_width = accumulate_price_axis_width_contribution(
            required_width,
            self.estimate_price_axis_tick_labels_required_width(
                style,
                selected_ticks,
                fallback_display_base_price,
                display_tick_step_abs,
                display_suffix,
            ),
        );

        required_width = accumulate_price_axis_width_contribution(
            required_width,
            self.estimate_last_price_axis_label_required_width(
                style,
                visible_start,
                visible_end,
                fallback_display_base_price,
                display_tick_step_abs,
                display_suffix,
            ),
        );

        required_width
    }
}
