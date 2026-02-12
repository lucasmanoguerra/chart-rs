use crate::render::Renderer;

use super::{
    ChartEngine, CrosshairFormatterOverrideMode, CrosshairPriceLabelFormatterWithContextFn,
    CrosshairTimeLabelFormatterWithContextFn, PriceLabelCacheStats, PriceLabelFormatterFn,
    TimeLabelCacheStats, TimeLabelFormatterFn,
};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn clear_crosshair_context_formatter_caches_if_needed(&self) {
        if self.crosshair_time_label_formatter_with_context.is_some() {
            self.crosshair_time_label_cache.borrow_mut().clear();
        }
        if self.crosshair_price_label_formatter_with_context.is_some() {
            self.crosshair_price_label_cache.borrow_mut().clear();
        }
    }

    pub fn set_time_label_formatter(&mut self, formatter: TimeLabelFormatterFn) {
        self.time_label_formatter = Some(formatter);
        self.time_label_formatter_generation =
            self.time_label_formatter_generation.saturating_add(1);
        self.time_label_cache.borrow_mut().clear();
    }

    pub fn clear_time_label_formatter(&mut self) {
        self.time_label_formatter = None;
        self.time_label_formatter_generation =
            self.time_label_formatter_generation.saturating_add(1);
        self.time_label_cache.borrow_mut().clear();
    }

    pub fn set_price_label_formatter(&mut self, formatter: PriceLabelFormatterFn) {
        self.price_label_formatter = Some(formatter);
        self.price_label_formatter_generation =
            self.price_label_formatter_generation.saturating_add(1);
        self.price_label_cache.borrow_mut().clear();
    }

    pub fn clear_price_label_formatter(&mut self) {
        self.price_label_formatter = None;
        self.price_label_formatter_generation =
            self.price_label_formatter_generation.saturating_add(1);
        self.price_label_cache.borrow_mut().clear();
    }

    /// Sets a formatter override used only for crosshair time-axis label text.
    pub fn set_crosshair_time_label_formatter(&mut self, formatter: TimeLabelFormatterFn) {
        self.crosshair_time_label_formatter = Some(formatter);
        self.crosshair_time_label_formatter_with_context = None;
        self.crosshair_time_label_formatter_generation = self
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.crosshair_time_label_cache.borrow_mut().clear();
    }

    /// Clears the crosshair time-axis label formatter override.
    pub fn clear_crosshair_time_label_formatter(&mut self) {
        self.crosshair_time_label_formatter = None;
        self.crosshair_time_label_formatter_with_context = None;
        self.crosshair_time_label_formatter_generation = self
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.crosshair_time_label_cache.borrow_mut().clear();
    }

    /// Sets a formatter override used only for crosshair price-axis label text.
    pub fn set_crosshair_price_label_formatter(&mut self, formatter: PriceLabelFormatterFn) {
        self.crosshair_price_label_formatter = Some(formatter);
        self.crosshair_price_label_formatter_with_context = None;
        self.crosshair_price_label_formatter_generation = self
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.crosshair_price_label_cache.borrow_mut().clear();
    }

    /// Clears the crosshair price-axis label formatter override.
    pub fn clear_crosshair_price_label_formatter(&mut self) {
        self.crosshair_price_label_formatter = None;
        self.crosshair_price_label_formatter_with_context = None;
        self.crosshair_price_label_formatter_generation = self
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.crosshair_price_label_cache.borrow_mut().clear();
    }

    /// Sets a context-aware formatter override used only for crosshair time-axis labels.
    pub fn set_crosshair_time_label_formatter_with_context(
        &mut self,
        formatter: CrosshairTimeLabelFormatterWithContextFn,
    ) {
        self.crosshair_time_label_formatter_with_context = Some(formatter);
        self.crosshair_time_label_formatter = None;
        self.crosshair_time_label_formatter_generation = self
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.crosshair_time_label_cache.borrow_mut().clear();
    }

    /// Clears the context-aware crosshair time-axis formatter override.
    pub fn clear_crosshair_time_label_formatter_with_context(&mut self) {
        self.crosshair_time_label_formatter_with_context = None;
        self.crosshair_time_label_formatter_generation = self
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.crosshair_time_label_cache.borrow_mut().clear();
    }

    /// Sets a context-aware formatter override used only for crosshair price-axis labels.
    pub fn set_crosshair_price_label_formatter_with_context(
        &mut self,
        formatter: CrosshairPriceLabelFormatterWithContextFn,
    ) {
        self.crosshair_price_label_formatter_with_context = Some(formatter);
        self.crosshair_price_label_formatter = None;
        self.crosshair_price_label_formatter_generation = self
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.crosshair_price_label_cache.borrow_mut().clear();
    }

    /// Clears the context-aware crosshair price-axis formatter override.
    pub fn clear_crosshair_price_label_formatter_with_context(&mut self) {
        self.crosshair_price_label_formatter_with_context = None;
        self.crosshair_price_label_formatter_generation = self
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.crosshair_price_label_cache.borrow_mut().clear();
    }

    #[must_use]
    pub fn crosshair_time_label_formatter_override_mode(&self) -> CrosshairFormatterOverrideMode {
        if self.crosshair_time_label_formatter_with_context.is_some() {
            CrosshairFormatterOverrideMode::Context
        } else if self.crosshair_time_label_formatter.is_some() {
            CrosshairFormatterOverrideMode::Legacy
        } else {
            CrosshairFormatterOverrideMode::None
        }
    }

    #[must_use]
    pub fn crosshair_price_label_formatter_override_mode(&self) -> CrosshairFormatterOverrideMode {
        if self.crosshair_price_label_formatter_with_context.is_some() {
            CrosshairFormatterOverrideMode::Context
        } else if self.crosshair_price_label_formatter.is_some() {
            CrosshairFormatterOverrideMode::Legacy
        } else {
            CrosshairFormatterOverrideMode::None
        }
    }

    #[must_use]
    pub fn crosshair_label_formatter_generations(&self) -> (u64, u64) {
        (
            self.crosshair_time_label_formatter_generation,
            self.crosshair_price_label_formatter_generation,
        )
    }

    #[must_use]
    pub fn crosshair_time_label_cache_stats(&self) -> TimeLabelCacheStats {
        self.crosshair_time_label_cache.borrow().stats()
    }

    pub fn clear_crosshair_time_label_cache(&self) {
        self.crosshair_time_label_cache.borrow_mut().clear();
    }

    #[must_use]
    pub fn crosshair_price_label_cache_stats(&self) -> PriceLabelCacheStats {
        self.crosshair_price_label_cache.borrow().stats()
    }

    pub fn clear_crosshair_price_label_cache(&self) {
        self.crosshair_price_label_cache.borrow_mut().clear();
    }

    #[must_use]
    pub fn time_label_cache_stats(&self) -> TimeLabelCacheStats {
        self.time_label_cache.borrow().stats()
    }

    pub fn clear_time_label_cache(&self) {
        self.time_label_cache.borrow_mut().clear();
    }

    /// Returns hit/miss counters for the price-axis label cache.
    #[must_use]
    pub fn price_label_cache_stats(&self) -> PriceLabelCacheStats {
        self.price_label_cache.borrow().stats()
    }

    /// Clears cached price-axis label strings.
    pub fn clear_price_label_cache(&self) {
        self.price_label_cache.borrow_mut().clear();
    }
}
