use crate::error::ChartResult;
use crate::render::Renderer;

use super::axis_price_width_accumulator::accumulate_required_price_axis_width;
use super::axis_price_width_bounds_resolver::{
    finalize_required_price_axis_width, initialize_required_price_axis_width,
};
use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn estimate_required_price_axis_width(
        &self,
        style: RenderStyle,
        plot_bottom: f64,
        visible_start: f64,
        visible_end: f64,
    ) -> ChartResult<f64> {
        let base_width = initialize_required_price_axis_width(style);
        let contribution_width = self.resolve_price_axis_width_contribution_from_pipeline(
            style,
            plot_bottom,
            visible_start,
            visible_end,
        )?;
        let required_width = accumulate_required_price_axis_width(base_width, contribution_width);

        Ok(finalize_required_price_axis_width(required_width))
    }
}
