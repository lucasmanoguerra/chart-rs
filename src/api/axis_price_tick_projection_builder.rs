use crate::error::ChartResult;
use crate::render::Renderer;

use super::ChartEngine;
use super::axis_ticks::tick_step_hint_from_values;

#[derive(Debug, Clone)]
pub(super) struct ProjectedPriceTicks {
    pub ticks: Vec<(f64, f64)>,
    pub tick_step_abs: f64,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn build_projected_price_ticks(
        &self,
        price_tick_count: usize,
        plot_bottom: f64,
    ) -> ChartResult<ProjectedPriceTicks> {
        let raw_price_ticks = self.core.model.price_scale.ticks(price_tick_count)?;
        let tick_step_abs = tick_step_hint_from_values(&raw_price_ticks);

        let mut ticks = Vec::with_capacity(raw_price_ticks.len());
        for price in raw_price_ticks.iter().copied() {
            let py = self
                .core
                .model
                .price_scale
                .price_to_pixel(price, self.core.model.viewport)?;
            let clamped_py = py.clamp(0.0, plot_bottom);
            ticks.push((price, clamped_py));
        }

        Ok(ProjectedPriceTicks {
            ticks,
            tick_step_abs,
        })
    }
}
