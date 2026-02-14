use crate::render::Renderer;

use super::axis_label_format::{
    ResolvedTimeLabelPattern, format_price_axis_label, format_price_axis_label_with_precision,
    format_time_axis_label, format_time_axis_label_with_precision, format_time_axis_tick_label,
    quantize_logical_time_millis, quantize_price_label_value, resolve_time_axis_tick_pattern,
};
use super::label_cache::{PriceLabelCacheKey, TimeLabelCacheKey, TimeLabelCacheProfile};
use super::{
    ChartEngine, CrosshairLabelSourceMode, CrosshairPriceLabelFormatterContext,
    CrosshairTimeLabelFormatterContext,
};

impl<R: Renderer> ChartEngine<R> {
    fn crosshair_source_mode_tag(source_mode: CrosshairLabelSourceMode) -> u8 {
        match source_mode {
            CrosshairLabelSourceMode::SnappedData => 1,
            CrosshairLabelSourceMode::PointerProjected => 2,
        }
    }

    fn quantize_visible_span_millis(visible_span_abs: f64) -> i64 {
        if !visible_span_abs.is_finite() {
            return 0;
        }
        let millis = (visible_span_abs.abs() * 1_000.0).round();
        if millis > i64::MAX as f64 {
            i64::MAX
        } else if millis < i64::MIN as f64 {
            i64::MIN
        } else {
            millis as i64
        }
    }

    pub(super) fn apply_crosshair_label_text_transform(
        text: String,
        prefix: &str,
        suffix: &str,
    ) -> String {
        if prefix.is_empty() && suffix.is_empty() {
            return text;
        }
        let mut transformed = String::with_capacity(prefix.len() + text.len() + suffix.len());
        transformed.push_str(prefix);
        transformed.push_str(&text);
        transformed.push_str(suffix);
        transformed
    }

    fn format_time_axis_label(&self, logical_time: f64, visible_span_abs: f64) -> String {
        let profile = self.resolve_time_label_cache_profile(visible_span_abs);
        let key = TimeLabelCacheKey {
            profile,
            logical_time_millis: quantize_logical_time_millis(logical_time),
        };

        if let Some(cached) = self
            .core
            .presentation
            .time_label_cache
            .borrow_mut()
            .get(key)
        {
            return cached;
        }

        let value = if let Some(formatter) = &self.core.presentation.time_label_formatter {
            formatter(logical_time)
        } else {
            format_time_axis_label(
                logical_time,
                self.core.behavior.time_axis_label_config,
                visible_span_abs,
            )
        };
        self.core
            .presentation
            .time_label_cache
            .borrow_mut()
            .insert(key, value.clone());
        value
    }

    pub(super) fn format_time_axis_tick_label(
        &self,
        logical_time: f64,
        visible_span_abs: f64,
        tick_step_abs: f64,
        is_major_tick: bool,
    ) -> String {
        let profile = if self.core.presentation.time_label_formatter.is_some() {
            TimeLabelCacheProfile::Custom {
                formatter_generation: self.core.presentation.time_label_formatter_generation,
                source_mode_tag: 0,
                visible_span_millis: 0,
            }
        } else {
            match resolve_time_axis_tick_pattern(
                self.core.behavior.time_axis_label_config.policy,
                visible_span_abs,
                tick_step_abs,
                is_major_tick,
            ) {
                ResolvedTimeLabelPattern::LogicalDecimal { precision } => {
                    TimeLabelCacheProfile::LogicalDecimal {
                        precision,
                        locale: self.core.behavior.time_axis_label_config.locale,
                    }
                }
                ResolvedTimeLabelPattern::Utc { pattern } => TimeLabelCacheProfile::Utc {
                    locale: self.core.behavior.time_axis_label_config.locale,
                    pattern,
                    timezone: self.core.behavior.time_axis_label_config.timezone,
                    session: self.core.behavior.time_axis_label_config.session,
                },
            }
        };
        let key = TimeLabelCacheKey {
            profile,
            logical_time_millis: quantize_logical_time_millis(logical_time),
        };
        if let Some(cached) = self
            .core
            .presentation
            .time_label_cache
            .borrow_mut()
            .get(key)
        {
            return cached;
        }

        let value = if let Some(formatter) = &self.core.presentation.time_label_formatter {
            formatter(logical_time)
        } else {
            format_time_axis_tick_label(
                logical_time,
                self.core.behavior.time_axis_label_config,
                visible_span_abs,
                tick_step_abs,
                is_major_tick,
            )
        };
        self.core
            .presentation
            .time_label_cache
            .borrow_mut()
            .insert(key, value.clone());
        value
    }

    pub(super) fn format_price_axis_label(
        &self,
        display_price: f64,
        tick_step_abs: f64,
        mode_suffix: &str,
    ) -> String {
        let profile = self.resolve_price_label_cache_profile();
        let key = PriceLabelCacheKey {
            profile,
            display_price_nanos: quantize_price_label_value(display_price),
            tick_step_nanos: quantize_price_label_value(tick_step_abs),
            has_percent_suffix: !mode_suffix.is_empty(),
        };

        if let Some(cached) = self
            .core
            .presentation
            .price_label_cache
            .borrow_mut()
            .get(key)
        {
            return cached;
        }

        let mut text = if let Some(formatter) = &self.core.presentation.price_label_formatter {
            formatter(display_price)
        } else {
            format_price_axis_label(
                display_price,
                self.core.behavior.price_axis_label_config,
                tick_step_abs,
            )
        };
        if !mode_suffix.is_empty() {
            text.push_str(mode_suffix);
        }
        self.core
            .presentation
            .price_label_cache
            .borrow_mut()
            .insert(key, text.clone());
        text
    }

    pub(super) fn format_crosshair_time_axis_label(
        &self,
        logical_time: f64,
        visible_span_abs: f64,
        precision_override: Option<u8>,
        source_mode: CrosshairLabelSourceMode,
    ) -> String {
        if let Some(formatter) = &self
            .core
            .presentation
            .crosshair_time_label_formatter_with_context
        {
            let key = TimeLabelCacheKey {
                profile: super::label_cache::TimeLabelCacheProfile::Custom {
                    formatter_generation: self
                        .core
                        .presentation
                        .crosshair_time_label_formatter_generation,
                    source_mode_tag: Self::crosshair_source_mode_tag(source_mode),
                    visible_span_millis: Self::quantize_visible_span_millis(visible_span_abs),
                },
                logical_time_millis: quantize_logical_time_millis(logical_time),
            };
            if let Some(cached) = self
                .core
                .presentation
                .crosshair_time_label_cache
                .borrow_mut()
                .get(key)
            {
                return cached;
            }
            let value = formatter(
                logical_time,
                CrosshairTimeLabelFormatterContext {
                    visible_span_abs,
                    source_mode,
                },
            );
            self.core
                .presentation
                .crosshair_time_label_cache
                .borrow_mut()
                .insert(key, value.clone());
            value
        } else if let Some(formatter) = &self.core.presentation.crosshair_time_label_formatter {
            let key = TimeLabelCacheKey {
                profile: super::label_cache::TimeLabelCacheProfile::Custom {
                    formatter_generation: self
                        .core
                        .presentation
                        .crosshair_time_label_formatter_generation,
                    source_mode_tag: 0,
                    visible_span_millis: 0,
                },
                logical_time_millis: quantize_logical_time_millis(logical_time),
            };
            if let Some(cached) = self
                .core
                .presentation
                .crosshair_time_label_cache
                .borrow_mut()
                .get(key)
            {
                return cached;
            }
            let value = formatter(logical_time);
            self.core
                .presentation
                .crosshair_time_label_cache
                .borrow_mut()
                .insert(key, value.clone());
            value
        } else if let Some(precision) = precision_override {
            format_time_axis_label_with_precision(
                logical_time,
                self.core.behavior.time_axis_label_config,
                visible_span_abs,
                precision,
            )
        } else {
            self.format_time_axis_label(logical_time, visible_span_abs)
        }
    }

    pub(super) fn format_crosshair_price_axis_label(
        &self,
        display_price: f64,
        tick_step_abs: f64,
        mode_suffix: &str,
        precision_override: Option<u8>,
        visible_span_abs: f64,
        source_mode: CrosshairLabelSourceMode,
    ) -> String {
        if let Some(formatter) = &self
            .core
            .presentation
            .crosshair_price_label_formatter_with_context
        {
            let key = PriceLabelCacheKey {
                profile: super::label_cache::PriceLabelCacheProfile::Custom {
                    formatter_generation: self
                        .core
                        .presentation
                        .crosshair_price_label_formatter_generation,
                    source_mode_tag: Self::crosshair_source_mode_tag(source_mode),
                    visible_span_millis: Self::quantize_visible_span_millis(visible_span_abs),
                },
                display_price_nanos: quantize_price_label_value(display_price),
                tick_step_nanos: quantize_price_label_value(tick_step_abs),
                has_percent_suffix: !mode_suffix.is_empty(),
            };
            if let Some(cached) = self
                .core
                .presentation
                .crosshair_price_label_cache
                .borrow_mut()
                .get(key)
            {
                return cached;
            }
            let mut value = formatter(
                display_price,
                CrosshairPriceLabelFormatterContext {
                    visible_span_abs,
                    source_mode,
                },
            );
            if !mode_suffix.is_empty() {
                value.push_str(mode_suffix);
            }
            self.core
                .presentation
                .crosshair_price_label_cache
                .borrow_mut()
                .insert(key, value.clone());
            value
        } else if let Some(formatter) = &self.core.presentation.crosshair_price_label_formatter {
            let key = PriceLabelCacheKey {
                profile: super::label_cache::PriceLabelCacheProfile::Custom {
                    formatter_generation: self
                        .core
                        .presentation
                        .crosshair_price_label_formatter_generation,
                    source_mode_tag: 0,
                    visible_span_millis: 0,
                },
                display_price_nanos: quantize_price_label_value(display_price),
                tick_step_nanos: quantize_price_label_value(tick_step_abs),
                has_percent_suffix: !mode_suffix.is_empty(),
            };
            if let Some(cached) = self
                .core
                .presentation
                .crosshair_price_label_cache
                .borrow_mut()
                .get(key)
            {
                return cached;
            }
            let mut value = formatter(display_price);
            if !mode_suffix.is_empty() {
                value.push_str(mode_suffix);
            }
            self.core
                .presentation
                .crosshair_price_label_cache
                .borrow_mut()
                .insert(key, value.clone());
            value
        } else if let Some(precision) = precision_override {
            let mut text = format_price_axis_label_with_precision(
                display_price,
                self.core.behavior.price_axis_label_config,
                tick_step_abs,
                precision,
            );
            if !mode_suffix.is_empty() {
                text.push_str(mode_suffix);
            }
            text
        } else {
            self.format_price_axis_label(display_price, tick_step_abs, mode_suffix)
        }
    }
}
