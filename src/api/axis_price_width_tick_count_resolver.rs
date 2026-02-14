use crate::error::ChartResult;
use crate::render::Renderer;

use super::ChartEngine;
use super::axis_ticks::{
    AXIS_PRICE_MIN_SPACING_PX, AXIS_PRICE_TARGET_SPACING_PX, axis_tick_target_count_with_density,
};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_price_axis_tick_count_for_width(
        &self,
        plot_bottom: f64,
    ) -> ChartResult<usize> {
        let price_density_scale = self.resolve_price_axis_density_scale();
        let price_axis_span_px = self.resolve_price_axis_span_px(plot_bottom)?;
        Ok(axis_tick_target_count_with_density(
            price_axis_span_px,
            AXIS_PRICE_TARGET_SPACING_PX,
            AXIS_PRICE_MIN_SPACING_PX,
            2,
            16,
            price_density_scale,
        ))
    }
}
