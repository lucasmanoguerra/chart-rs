use std::collections::HashMap;
use std::sync::Arc;

use super::{AxisLabelLocale, PriceAxisLabelPolicy, TimeAxisSessionConfig, TimeAxisTimeZone};

pub type TimeLabelFormatterFn = Arc<dyn Fn(f64) -> String + Send + Sync + 'static>;
pub type PriceLabelFormatterFn = Arc<dyn Fn(f64) -> String + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeLabelCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

/// Runtime metrics exposed by the in-engine price-label cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PriceLabelCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum TimeLabelPattern {
    Date,
    DateMinute,
    DateSecond,
    TimeMinute,
    TimeSecond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PriceLabelCachePolicy {
    FixedDecimals {
        precision: u8,
    },
    MinMove {
        min_move_nanos: i64,
        trim_trailing_zeros: bool,
    },
    Adaptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum TimeLabelCacheProfile {
    LogicalDecimal {
        precision: u8,
        locale: AxisLabelLocale,
    },
    Utc {
        locale: AxisLabelLocale,
        pattern: TimeLabelPattern,
        timezone: TimeAxisTimeZone,
        session: Option<TimeAxisSessionConfig>,
    },
    Custom {
        formatter_generation: u64,
        source_mode_tag: u8,
        visible_span_millis: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PriceLabelCacheProfile {
    BuiltIn {
        locale: AxisLabelLocale,
        policy: PriceLabelCachePolicy,
    },
    Custom {
        formatter_generation: u64,
        source_mode_tag: u8,
        visible_span_millis: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct TimeLabelCacheKey {
    pub(super) profile: TimeLabelCacheProfile,
    pub(super) logical_time_millis: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct PriceLabelCacheKey {
    pub(super) profile: PriceLabelCacheProfile,
    pub(super) display_price_nanos: i64,
    pub(super) tick_step_nanos: i64,
    pub(super) has_percent_suffix: bool,
}

#[derive(Debug, Default)]
pub(super) struct TimeLabelCache {
    entries: HashMap<TimeLabelCacheKey, String>,
    hits: u64,
    misses: u64,
}

#[derive(Debug, Default)]
pub(super) struct PriceLabelCache {
    entries: HashMap<PriceLabelCacheKey, String>,
    hits: u64,
    misses: u64,
}

impl TimeLabelCache {
    const MAX_ENTRIES: usize = 8192;

    pub(super) fn get(&mut self, key: TimeLabelCacheKey) -> Option<String> {
        let value = self.entries.get(&key).cloned();
        if value.is_some() {
            self.hits = self.hits.saturating_add(1);
        }
        value
    }

    pub(super) fn insert(&mut self, key: TimeLabelCacheKey, value: String) {
        self.misses = self.misses.saturating_add(1);
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.clear();
        }
        self.entries.insert(key, value);
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(super) fn stats(&self) -> TimeLabelCacheStats {
        TimeLabelCacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.entries.len(),
        }
    }
}

impl PriceLabelCache {
    const MAX_ENTRIES: usize = 8192;

    pub(super) fn get(&mut self, key: PriceLabelCacheKey) -> Option<String> {
        let value = self.entries.get(&key).cloned();
        if value.is_some() {
            self.hits = self.hits.saturating_add(1);
        }
        value
    }

    pub(super) fn insert(&mut self, key: PriceLabelCacheKey, value: String) {
        self.misses = self.misses.saturating_add(1);
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.clear();
        }
        self.entries.insert(key, value);
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(super) fn stats(&self) -> PriceLabelCacheStats {
        PriceLabelCacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.entries.len(),
        }
    }
}

pub(super) fn price_policy_profile(policy: PriceAxisLabelPolicy) -> PriceLabelCachePolicy {
    match policy {
        PriceAxisLabelPolicy::FixedDecimals { precision } => {
            PriceLabelCachePolicy::FixedDecimals { precision }
        }
        PriceAxisLabelPolicy::MinMove {
            min_move,
            trim_trailing_zeros,
        } => PriceLabelCachePolicy::MinMove {
            min_move_nanos: quantize_price_label_value(min_move),
            trim_trailing_zeros,
        },
        PriceAxisLabelPolicy::Adaptive => PriceLabelCachePolicy::Adaptive,
    }
}

fn quantize_price_label_value(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    let nanos = (value * 1_000_000_000.0).round();
    if nanos > (i64::MAX as f64) {
        i64::MAX
    } else if nanos < (i64::MIN as f64) {
        i64::MIN
    } else {
        nanos as i64
    }
}
