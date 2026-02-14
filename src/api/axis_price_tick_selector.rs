use crate::error::ChartResult;
use crate::render::Renderer;

use super::axis_price_tick_exclusion_filter::filter_price_ticks_for_last_price_label;
use super::axis_price_tick_spacing_selector::select_price_ticks_with_min_spacing;
use super::last_price_axis_scene_builder::LastPriceMarker;
use super::{ChartEngine, RenderStyle};

#[derive(Debug, Clone)]
pub(super) struct PriceAxisTickSelection {
    pub ticks: Vec<(f64, f64)>,
    pub tick_step_abs: f64,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn select_price_axis_ticks(
        &self,
        price_tick_count: usize,
        plot_bottom: f64,
        style: RenderStyle,
        latest_price_marker: Option<LastPriceMarker>,
    ) -> ChartResult<PriceAxisTickSelection> {
        let projected_ticks = self.build_projected_price_ticks(price_tick_count, plot_bottom)?;
        let tick_step_abs = projected_ticks.tick_step_abs;
        let price_ticks = projected_ticks.ticks;

        let selected_price_ticks = select_price_ticks_with_min_spacing(price_ticks);
        let ticks = filter_price_ticks_for_last_price_label(
            &selected_price_ticks,
            style,
            latest_price_marker,
        );

        Ok(PriceAxisTickSelection {
            ticks,
            tick_step_abs,
        })
    }
}
