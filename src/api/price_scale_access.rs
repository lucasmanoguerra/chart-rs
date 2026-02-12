use crate::core::{PriceScale, PriceScaleMode, PriceScaleTuning};
use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{ChartEngine, PriceScaleMarginBehavior, PriceScaleRealtimeBehavior};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn price_scale_realtime_behavior(&self) -> PriceScaleRealtimeBehavior {
        self.price_scale_realtime_behavior
    }

    pub fn set_price_scale_realtime_behavior(&mut self, behavior: PriceScaleRealtimeBehavior) {
        self.price_scale_realtime_behavior = behavior;
    }

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

    /// Returns whether price-axis pixel mapping is inverted.
    #[must_use]
    pub fn price_scale_inverted(&self) -> bool {
        self.price_scale.is_inverted()
    }

    /// Enables/disables inverted price-axis mapping.
    pub fn set_price_scale_inverted(&mut self, inverted: bool) {
        self.price_scale = self.price_scale.with_inverted(inverted);
    }

    #[must_use]
    pub fn price_scale_margin_behavior(&self) -> PriceScaleMarginBehavior {
        let (top_margin_ratio, bottom_margin_ratio) = self.price_scale.margins();
        PriceScaleMarginBehavior {
            top_margin_ratio,
            bottom_margin_ratio,
        }
    }

    pub fn set_price_scale_margin_behavior(
        &mut self,
        behavior: PriceScaleMarginBehavior,
    ) -> ChartResult<()> {
        if !behavior.top_margin_ratio.is_finite()
            || !behavior.bottom_margin_ratio.is_finite()
            || behavior.top_margin_ratio < 0.0
            || behavior.bottom_margin_ratio < 0.0
        {
            return Err(ChartError::InvalidData(
                "price scale margins must be finite and >= 0".to_owned(),
            ));
        }
        if behavior.top_margin_ratio + behavior.bottom_margin_ratio >= 1.0 {
            return Err(ChartError::InvalidData(
                "price scale margins must sum to < 1".to_owned(),
            ));
        }
        self.price_scale = self
            .price_scale
            .with_margins(behavior.top_margin_ratio, behavior.bottom_margin_ratio)?;
        Ok(())
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
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale =
            PriceScale::from_data_tuned_with_mode(&self.points, tuning, self.price_scale_mode)?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
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
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale =
            PriceScale::from_ohlc_tuned_with_mode(&self.candles, tuning, self.price_scale_mode)?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }

    /// Autoscales price domain from currently visible points.
    pub fn autoscale_price_from_visible_data(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_visible_data_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from visible points with explicit tuning.
    pub fn autoscale_price_from_visible_data_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        let visible = self.visible_points();
        if visible.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale =
            PriceScale::from_data_tuned_with_mode(&visible, tuning, self.price_scale_mode)?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }

    /// Autoscales price domain from currently visible candles.
    pub fn autoscale_price_from_visible_candles(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_visible_candles_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from visible candles with explicit tuning.
    pub fn autoscale_price_from_visible_candles_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        let visible = self.visible_candles();
        if visible.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale =
            PriceScale::from_ohlc_tuned_with_mode(&visible, tuning, self.price_scale_mode)?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }
}
