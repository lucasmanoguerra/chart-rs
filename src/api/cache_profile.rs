use crate::render::Renderer;

use super::ChartEngine;
use super::axis_label_format::{ResolvedTimeLabelPattern, resolve_time_label_pattern};
use super::label_cache::{PriceLabelCacheProfile, TimeLabelCacheProfile, price_policy_profile};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_time_label_cache_profile(
        &self,
        visible_span_abs: f64,
    ) -> TimeLabelCacheProfile {
        if self.core.presentation.time_label_formatter.is_some() {
            return TimeLabelCacheProfile::Custom {
                formatter_generation: self.core.presentation.time_label_formatter_generation,
                source_mode_tag: 0,
                visible_span_millis: 0,
            };
        }

        match resolve_time_label_pattern(
            self.core.behavior.time_axis_label_config.policy,
            visible_span_abs,
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
    }

    pub(super) fn resolve_price_label_cache_profile(&self) -> PriceLabelCacheProfile {
        if self.core.presentation.price_label_formatter.is_some() {
            return PriceLabelCacheProfile::Custom {
                formatter_generation: self.core.presentation.price_label_formatter_generation,
                source_mode_tag: 0,
                visible_span_millis: 0,
            };
        }

        PriceLabelCacheProfile::BuiltIn {
            locale: self.core.behavior.price_axis_label_config.locale,
            policy: price_policy_profile(self.core.behavior.price_axis_label_config.policy),
        }
    }
}
