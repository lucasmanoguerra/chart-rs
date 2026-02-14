use crate::error::ChartResult;
use crate::render::Renderer;

use super::ChartEngine;
use super::axis_price_tick_spacing_selector::select_price_ticks_with_min_spacing;
use super::axis_render_frame_builder::AxisPriceDisplayContext;

#[derive(Debug, Clone)]
pub(super) struct PriceAxisWidthTickContext {
    pub selected_ticks: Vec<(f64, f64)>,
    pub display_context: AxisPriceDisplayContext,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_price_axis_width_tick_context(
        &self,
        price_tick_count: usize,
        plot_bottom: f64,
    ) -> ChartResult<PriceAxisWidthTickContext> {
        let projected_ticks = self.build_projected_price_ticks(price_tick_count, plot_bottom)?;
        let selected_ticks = select_price_ticks_with_min_spacing(projected_ticks.ticks);
        let display_context =
            self.resolve_price_axis_display_context(projected_ticks.tick_step_abs);

        Ok(PriceAxisWidthTickContext {
            selected_ticks,
            display_context,
        })
    }
}
