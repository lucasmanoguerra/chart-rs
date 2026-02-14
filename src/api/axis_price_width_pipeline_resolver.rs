use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_price_axis_width_contribution_from_pipeline(
        &self,
        style: RenderStyle,
        plot_bottom: f64,
        visible_start: f64,
        visible_end: f64,
    ) -> ChartResult<f64> {
        let price_tick_count = self.resolve_price_axis_tick_count_for_width(plot_bottom)?;
        let width_tick_context =
            self.resolve_price_axis_width_tick_context(price_tick_count, plot_bottom)?;
        Ok(self.estimate_price_axis_width_contribution(
            style,
            visible_start,
            visible_end,
            &width_tick_context,
        ))
    }
}
