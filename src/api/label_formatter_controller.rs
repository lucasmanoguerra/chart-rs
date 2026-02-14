use crate::render::Renderer;

use super::{
    ChartEngine, CrosshairFormatterDiagnostics, CrosshairFormatterOverrideMode,
    CrosshairPriceLabelFormatterWithContextFn, CrosshairTimeLabelFormatterWithContextFn,
    PriceLabelCacheStats, PriceLabelFormatterFn, TimeLabelCacheStats, TimeLabelFormatterFn,
};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn clear_crosshair_context_formatter_caches_if_needed(&self) {
        if self
            .core
            .presentation
            .crosshair_time_label_formatter_with_context
            .is_some()
        {
            self.core
                .presentation
                .crosshair_time_label_cache
                .borrow_mut()
                .clear();
        }
        if self
            .core
            .presentation
            .crosshair_price_label_formatter_with_context
            .is_some()
        {
            self.core
                .presentation
                .crosshair_price_label_cache
                .borrow_mut()
                .clear();
        }
    }

    pub fn set_time_label_formatter(&mut self, formatter: TimeLabelFormatterFn) {
        self.core.presentation.time_label_formatter = Some(formatter);
        self.core.presentation.time_label_formatter_generation = self
            .core
            .presentation
            .time_label_formatter_generation
            .saturating_add(1);
        self.core.presentation.time_label_cache.borrow_mut().clear();
    }

    pub fn clear_time_label_formatter(&mut self) {
        self.core.presentation.time_label_formatter = None;
        self.core.presentation.time_label_formatter_generation = self
            .core
            .presentation
            .time_label_formatter_generation
            .saturating_add(1);
        self.core.presentation.time_label_cache.borrow_mut().clear();
    }

    pub fn set_price_label_formatter(&mut self, formatter: PriceLabelFormatterFn) {
        self.core.presentation.price_label_formatter = Some(formatter);
        self.core.presentation.price_label_formatter_generation = self
            .core
            .presentation
            .price_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .price_label_cache
            .borrow_mut()
            .clear();
    }

    pub fn clear_price_label_formatter(&mut self) {
        self.core.presentation.price_label_formatter = None;
        self.core.presentation.price_label_formatter_generation = self
            .core
            .presentation
            .price_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .price_label_cache
            .borrow_mut()
            .clear();
    }

    /// Sets a formatter override used only for crosshair time-axis label text.
    pub fn set_crosshair_time_label_formatter(&mut self, formatter: TimeLabelFormatterFn) {
        self.core.presentation.crosshair_time_label_formatter = Some(formatter);
        self.core
            .presentation
            .crosshair_time_label_formatter_with_context = None;
        self.core
            .presentation
            .crosshair_time_label_formatter_generation = self
            .core
            .presentation
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_time_label_cache
            .borrow_mut()
            .clear();
    }

    /// Clears the crosshair time-axis label formatter override.
    pub fn clear_crosshair_time_label_formatter(&mut self) {
        self.core.presentation.crosshair_time_label_formatter = None;
        self.core
            .presentation
            .crosshair_time_label_formatter_with_context = None;
        self.core
            .presentation
            .crosshair_time_label_formatter_generation = self
            .core
            .presentation
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_time_label_cache
            .borrow_mut()
            .clear();
    }

    /// Sets a formatter override used only for crosshair price-axis label text.
    pub fn set_crosshair_price_label_formatter(&mut self, formatter: PriceLabelFormatterFn) {
        self.core.presentation.crosshair_price_label_formatter = Some(formatter);
        self.core
            .presentation
            .crosshair_price_label_formatter_with_context = None;
        self.core
            .presentation
            .crosshair_price_label_formatter_generation = self
            .core
            .presentation
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_price_label_cache
            .borrow_mut()
            .clear();
    }

    /// Clears the crosshair price-axis label formatter override.
    pub fn clear_crosshair_price_label_formatter(&mut self) {
        self.core.presentation.crosshair_price_label_formatter = None;
        self.core
            .presentation
            .crosshair_price_label_formatter_with_context = None;
        self.core
            .presentation
            .crosshair_price_label_formatter_generation = self
            .core
            .presentation
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_price_label_cache
            .borrow_mut()
            .clear();
    }

    /// Sets a context-aware formatter override used only for crosshair time-axis labels.
    pub fn set_crosshair_time_label_formatter_with_context(
        &mut self,
        formatter: CrosshairTimeLabelFormatterWithContextFn,
    ) {
        self.core
            .presentation
            .crosshair_time_label_formatter_with_context = Some(formatter);
        self.core.presentation.crosshair_time_label_formatter = None;
        self.core
            .presentation
            .crosshair_time_label_formatter_generation = self
            .core
            .presentation
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_time_label_cache
            .borrow_mut()
            .clear();
    }

    /// Clears the context-aware crosshair time-axis formatter override.
    pub fn clear_crosshair_time_label_formatter_with_context(&mut self) {
        self.core
            .presentation
            .crosshair_time_label_formatter_with_context = None;
        self.core
            .presentation
            .crosshair_time_label_formatter_generation = self
            .core
            .presentation
            .crosshair_time_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_time_label_cache
            .borrow_mut()
            .clear();
    }

    /// Sets a context-aware formatter override used only for crosshair price-axis labels.
    pub fn set_crosshair_price_label_formatter_with_context(
        &mut self,
        formatter: CrosshairPriceLabelFormatterWithContextFn,
    ) {
        self.core
            .presentation
            .crosshair_price_label_formatter_with_context = Some(formatter);
        self.core.presentation.crosshair_price_label_formatter = None;
        self.core
            .presentation
            .crosshair_price_label_formatter_generation = self
            .core
            .presentation
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_price_label_cache
            .borrow_mut()
            .clear();
    }

    /// Clears the context-aware crosshair price-axis formatter override.
    pub fn clear_crosshair_price_label_formatter_with_context(&mut self) {
        self.core
            .presentation
            .crosshair_price_label_formatter_with_context = None;
        self.core
            .presentation
            .crosshair_price_label_formatter_generation = self
            .core
            .presentation
            .crosshair_price_label_formatter_generation
            .saturating_add(1);
        self.core
            .presentation
            .crosshair_price_label_cache
            .borrow_mut()
            .clear();
    }

    #[must_use]
    pub fn crosshair_time_label_formatter_override_mode(&self) -> CrosshairFormatterOverrideMode {
        if self
            .core
            .presentation
            .crosshair_time_label_formatter_with_context
            .is_some()
        {
            CrosshairFormatterOverrideMode::Context
        } else if self
            .core
            .presentation
            .crosshair_time_label_formatter
            .is_some()
        {
            CrosshairFormatterOverrideMode::Legacy
        } else {
            CrosshairFormatterOverrideMode::None
        }
    }

    #[must_use]
    pub fn crosshair_price_label_formatter_override_mode(&self) -> CrosshairFormatterOverrideMode {
        if self
            .core
            .presentation
            .crosshair_price_label_formatter_with_context
            .is_some()
        {
            CrosshairFormatterOverrideMode::Context
        } else if self
            .core
            .presentation
            .crosshair_price_label_formatter
            .is_some()
        {
            CrosshairFormatterOverrideMode::Legacy
        } else {
            CrosshairFormatterOverrideMode::None
        }
    }

    #[must_use]
    pub fn crosshair_label_formatter_generations(&self) -> (u64, u64) {
        (
            self.core
                .presentation
                .crosshair_time_label_formatter_generation,
            self.core
                .presentation
                .crosshair_price_label_formatter_generation,
        )
    }

    #[must_use]
    pub fn crosshair_formatter_diagnostics(&self) -> CrosshairFormatterDiagnostics {
        let (time_formatter_generation, price_formatter_generation) =
            self.crosshair_label_formatter_generations();
        CrosshairFormatterDiagnostics {
            time_override_mode: self.crosshair_time_label_formatter_override_mode(),
            price_override_mode: self.crosshair_price_label_formatter_override_mode(),
            time_formatter_generation,
            price_formatter_generation,
            time_cache: self.crosshair_time_label_cache_stats(),
            price_cache: self.crosshair_price_label_cache_stats(),
        }
    }

    pub fn clear_crosshair_formatter_caches(&self) {
        self.core
            .presentation
            .crosshair_time_label_cache
            .borrow_mut()
            .clear();
        self.core
            .presentation
            .crosshair_price_label_cache
            .borrow_mut()
            .clear();
    }

    #[must_use]
    pub fn crosshair_time_label_cache_stats(&self) -> TimeLabelCacheStats {
        self.core
            .presentation
            .crosshair_time_label_cache
            .borrow()
            .stats()
    }

    pub fn clear_crosshair_time_label_cache(&self) {
        self.core
            .presentation
            .crosshair_time_label_cache
            .borrow_mut()
            .clear();
    }

    #[must_use]
    pub fn crosshair_price_label_cache_stats(&self) -> PriceLabelCacheStats {
        self.core
            .presentation
            .crosshair_price_label_cache
            .borrow()
            .stats()
    }

    pub fn clear_crosshair_price_label_cache(&self) {
        self.core
            .presentation
            .crosshair_price_label_cache
            .borrow_mut()
            .clear();
    }

    #[must_use]
    pub fn time_label_cache_stats(&self) -> TimeLabelCacheStats {
        self.core.presentation.time_label_cache.borrow().stats()
    }

    pub fn clear_time_label_cache(&self) {
        self.core.presentation.time_label_cache.borrow_mut().clear();
    }

    /// Returns hit/miss counters for the price-axis label cache.
    #[must_use]
    pub fn price_label_cache_stats(&self) -> PriceLabelCacheStats {
        self.core.presentation.price_label_cache.borrow().stats()
    }

    /// Clears cached price-axis label strings.
    pub fn clear_price_label_cache(&self) {
        self.core
            .presentation
            .price_label_cache
            .borrow_mut()
            .clear();
    }
}
