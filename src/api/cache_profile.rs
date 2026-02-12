use crate::render::Renderer;

use super::{
    ChartEngine, PriceLabelCacheProfile, ResolvedTimeLabelPattern, TimeLabelCacheProfile,
    price_policy_profile, resolve_time_label_pattern,
};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_time_label_cache_profile(
        &self,
        visible_span_abs: f64,
    ) -> TimeLabelCacheProfile {
        if self.time_label_formatter.is_some() {
            return TimeLabelCacheProfile::Custom {
                formatter_generation: self.time_label_formatter_generation,
            };
        }

        match resolve_time_label_pattern(self.time_axis_label_config.policy, visible_span_abs) {
            ResolvedTimeLabelPattern::LogicalDecimal { precision } => {
                TimeLabelCacheProfile::LogicalDecimal {
                    precision,
                    locale: self.time_axis_label_config.locale,
                }
            }
            ResolvedTimeLabelPattern::Utc { pattern } => TimeLabelCacheProfile::Utc {
                locale: self.time_axis_label_config.locale,
                pattern,
                timezone: self.time_axis_label_config.timezone,
                session: self.time_axis_label_config.session,
            },
        }
    }

    pub(super) fn resolve_price_label_cache_profile(&self) -> PriceLabelCacheProfile {
        if self.price_label_formatter.is_some() {
            return PriceLabelCacheProfile::Custom {
                formatter_generation: self.price_label_formatter_generation,
            };
        }

        PriceLabelCacheProfile::BuiltIn {
            locale: self.price_axis_label_config.locale,
            policy: price_policy_profile(self.price_axis_label_config.policy),
        }
    }
}
