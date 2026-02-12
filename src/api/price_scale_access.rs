use crate::core::{PriceScale, PriceScaleMode, PriceScaleTuning};
use crate::error::ChartResult;
use crate::render::Renderer;

use super::ChartEngine;

impl<R: Renderer> ChartEngine<R> {
    /// Maps a raw price value into pixel Y under the active price scale mode.
    pub fn map_price_to_pixel(&self, price: f64) -> ChartResult<f64> {
        self.price_scale.price_to_pixel(price, self.viewport)
    }

    /// Maps a pixel Y coordinate back into a raw price value.
    pub fn map_pixel_to_price(&self, pixel: f64) -> ChartResult<f64> {
        self.price_scale.pixel_to_price(pixel, self.viewport)
    }

    #[must_use]
    pub fn price_domain(&self) -> (f64, f64) {
        self.price_scale.domain()
    }

    /// Returns the active price scale mapping mode.
    #[must_use]
    pub fn price_scale_mode(&self) -> PriceScaleMode {
        self.price_scale_mode
    }

    /// Switches the price scale mapping mode while preserving the current raw domain.
    ///
    /// When switching to `PriceScaleMode::Log`, the current domain must be strictly positive.
    pub fn set_price_scale_mode(&mut self, mode: PriceScaleMode) -> ChartResult<()> {
        self.price_scale = self.price_scale.with_mode(mode)?;
        self.price_scale_mode = mode;
        Ok(())
    }

    pub fn autoscale_price_from_data(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_data_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from points with explicit tuning.
    pub fn autoscale_price_from_data_tuned(&mut self, tuning: PriceScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() {
            return Ok(());
        }
        self.price_scale =
            PriceScale::from_data_tuned_with_mode(&self.points, tuning, self.price_scale_mode)?;
        Ok(())
    }

    pub fn autoscale_price_from_candles(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_candles_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from candles with explicit tuning.
    pub fn autoscale_price_from_candles_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        if self.candles.is_empty() {
            return Ok(());
        }
        self.price_scale =
            PriceScale::from_ohlc_tuned_with_mode(&self.candles, tuning, self.price_scale_mode)?;
        Ok(())
    }
}
