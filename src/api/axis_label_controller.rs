use crate::error::ChartResult;
use crate::render::Renderer;

use super::validation::{validate_price_axis_label_config, validate_time_axis_label_config};
use super::{ChartEngine, PriceAxisLabelConfig, TimeAxisLabelConfig};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn time_axis_label_config(&self) -> TimeAxisLabelConfig {
        self.core.behavior.time_axis_label_config
    }

    pub fn set_time_axis_label_config(&mut self, config: TimeAxisLabelConfig) -> ChartResult<()> {
        validate_time_axis_label_config(config)?;
        self.core.behavior.time_axis_label_config = config;
        self.core.presentation.time_label_cache.borrow_mut().clear();
        self.invalidate_axis();
        Ok(())
    }

    #[must_use]
    pub fn price_axis_label_config(&self) -> PriceAxisLabelConfig {
        self.core.behavior.price_axis_label_config
    }

    pub fn set_price_axis_label_config(&mut self, config: PriceAxisLabelConfig) -> ChartResult<()> {
        validate_price_axis_label_config(config)?;
        self.core.behavior.price_axis_label_config = config;
        self.core
            .presentation
            .price_label_cache
            .borrow_mut()
            .clear();
        self.invalidate_axis();
        Ok(())
    }
}
