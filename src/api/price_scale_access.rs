use crate::core::{PriceScale, PriceScaleMode, PriceScaleTuning};
use crate::error::ChartResult;
use crate::render::Renderer;

use super::{
    ChartEngine, PriceScaleMarginBehavior, PriceScaleRealtimeBehavior,
    PriceScaleTransformedBaseBehavior, price_scale_coordinator::PriceScaleCoordinator,
    price_scale_validation,
};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn price_scale_realtime_behavior(&self) -> PriceScaleRealtimeBehavior {
        self.core.behavior.price_scale_realtime_behavior
    }

    pub fn set_price_scale_realtime_behavior(&mut self, behavior: PriceScaleRealtimeBehavior) {
        self.core.behavior.price_scale_realtime_behavior = behavior;
    }

    /// Maps a raw price value into pixel Y under the active price scale mode.
    pub fn map_price_to_pixel(&self, price: f64) -> ChartResult<f64> {
        self.core
            .model
            .price_scale
            .price_to_pixel(price, self.core.model.viewport)
    }

    /// Maps a pixel Y coordinate back into a raw price value.
    pub fn map_pixel_to_price(&self, pixel: f64) -> ChartResult<f64> {
        self.core
            .model
            .price_scale
            .pixel_to_price(pixel, self.core.model.viewport)
    }

    #[must_use]
    pub fn price_domain(&self) -> (f64, f64) {
        self.core.model.price_scale.domain()
    }

    /// Returns the active price scale mapping mode.
    #[must_use]
    pub fn price_scale_mode(&self) -> PriceScaleMode {
        self.core.model.price_scale_mode
    }

    /// Returns whether price-axis pixel mapping is inverted.
    #[must_use]
    pub fn price_scale_inverted(&self) -> bool {
        self.core.model.price_scale.is_inverted()
    }

    #[must_use]
    pub fn price_scale_transformed_base_behavior(&self) -> PriceScaleTransformedBaseBehavior {
        self.core.behavior.price_scale_transformed_base_behavior
    }

    /// Returns current resolved transformed base value when active mode is
    /// `Percentage` or `IndexedTo100`.
    #[must_use]
    pub fn price_scale_transformed_base_value(&self) -> Option<f64> {
        self.core.model.price_scale.base_value()
    }

    /// Enables/disables inverted price-axis mapping.
    pub fn set_price_scale_inverted(&mut self, inverted: bool) {
        self.core.model.price_scale = self.core.model.price_scale.with_inverted(inverted);
        self.invalidate_price_scale();
    }

    pub fn set_price_scale_transformed_base_behavior(
        &mut self,
        behavior: PriceScaleTransformedBaseBehavior,
    ) -> ChartResult<()> {
        price_scale_validation::validate_price_scale_transformed_base_behavior(behavior)?;
        self.core.behavior.price_scale_transformed_base_behavior = behavior;
        let changed = self.refresh_price_scale_transformed_base()?;
        if !changed {
            self.invalidate_price_scale();
        }
        Ok(())
    }

    #[must_use]
    pub fn price_scale_margin_behavior(&self) -> PriceScaleMarginBehavior {
        let (top_margin_ratio, bottom_margin_ratio) = self.core.model.price_scale.margins();
        PriceScaleMarginBehavior {
            top_margin_ratio,
            bottom_margin_ratio,
        }
    }

    pub fn set_price_scale_margin_behavior(
        &mut self,
        behavior: PriceScaleMarginBehavior,
    ) -> ChartResult<()> {
        price_scale_validation::validate_price_scale_margin_behavior(behavior)?;
        self.core.model.price_scale = self
            .core
            .model
            .price_scale
            .with_margins(behavior.top_margin_ratio, behavior.bottom_margin_ratio)?;
        self.invalidate_price_scale();
        Ok(())
    }

    /// Switches the price scale mapping mode while preserving the current raw domain.
    ///
    /// When switching to `PriceScaleMode::Log`, the current domain must be strictly positive.
    /// Transformed display modes (`Percentage` / `IndexedTo100`) resolve a
    /// deterministic non-zero base value from the current domain.
    pub fn set_price_scale_mode(&mut self, mode: PriceScaleMode) -> ChartResult<()> {
        let base_value =
            PriceScaleCoordinator::resolve_price_scale_transformed_base_value(self, mode);
        self.core.model.price_scale = self
            .core
            .model
            .price_scale
            .with_mode_and_base(mode, base_value)?;
        self.core.model.price_scale_mode = mode;
        self.invalidate_price_scale();
        Ok(())
    }

    pub fn autoscale_price_from_data(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_data_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from points with explicit tuning.
    pub fn autoscale_price_from_data_tuned(&mut self, tuning: PriceScaleTuning) -> ChartResult<()> {
        if self.core.model.points.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.core.model.price_scale.is_inverted();
        let keep_margins = self.core.model.price_scale.margins();
        let base_value = PriceScaleCoordinator::resolve_price_scale_transformed_base_value(
            self,
            self.core.model.price_scale_mode,
        );
        self.core.model.price_scale = PriceScale::from_data_tuned_with_mode(
            &self.core.model.points,
            tuning,
            self.core.model.price_scale_mode,
        )?
        .with_base_value(base_value)?
        .with_inverted(keep_inverted)
        .with_margins(keep_margins.0, keep_margins.1)?;
        self.invalidate_price_scale();
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
        if self.core.model.candles.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.core.model.price_scale.is_inverted();
        let keep_margins = self.core.model.price_scale.margins();
        self.core.model.price_scale = PriceScale::from_ohlc_tuned_with_mode(
            &self.core.model.candles,
            tuning,
            self.core.model.price_scale_mode,
        )?
        .with_base_value(
            PriceScaleCoordinator::resolve_price_scale_transformed_base_value(
                self,
                self.core.model.price_scale_mode,
            ),
        )?
        .with_inverted(keep_inverted)
        .with_margins(keep_margins.0, keep_margins.1)?;
        self.invalidate_price_scale();
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
        let keep_inverted = self.core.model.price_scale.is_inverted();
        let keep_margins = self.core.model.price_scale.margins();
        self.core.model.price_scale = PriceScale::from_data_tuned_with_mode(
            &visible,
            tuning,
            self.core.model.price_scale_mode,
        )?
        .with_base_value(
            PriceScaleCoordinator::resolve_price_scale_transformed_base_value(
                self,
                self.core.model.price_scale_mode,
            ),
        )?
        .with_inverted(keep_inverted)
        .with_margins(keep_margins.0, keep_margins.1)?;
        self.invalidate_price_scale();
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
        let keep_inverted = self.core.model.price_scale.is_inverted();
        let keep_margins = self.core.model.price_scale.margins();
        self.core.model.price_scale = PriceScale::from_ohlc_tuned_with_mode(
            &visible,
            tuning,
            self.core.model.price_scale_mode,
        )?
        .with_base_value(
            PriceScaleCoordinator::resolve_price_scale_transformed_base_value(
                self,
                self.core.model.price_scale_mode,
            ),
        )?
        .with_inverted(keep_inverted)
        .with_margins(keep_margins.0, keep_margins.1)?;
        self.invalidate_price_scale();
        Ok(())
    }

    pub(crate) fn rebuild_price_scale_from_domain_preserving_mode(
        &mut self,
        domain_start: f64,
        domain_end: f64,
    ) -> ChartResult<()> {
        PriceScaleCoordinator::rebuild_price_scale_from_domain_preserving_mode(
            self,
            domain_start,
            domain_end,
        )
    }

    pub(crate) fn refresh_price_scale_transformed_base(&mut self) -> ChartResult<bool> {
        PriceScaleCoordinator::refresh_price_scale_transformed_base(self)
    }
}
