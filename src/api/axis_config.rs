use chrono::FixedOffset;
use serde::{Deserialize, Serialize};

/// Locale preset used by axis label formatters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AxisLabelLocale {
    #[default]
    EnUs,
    EsEs,
}

/// Built-in policy used for time-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeAxisLabelPolicy {
    /// Render logical time values as decimals.
    LogicalDecimal { precision: u8 },
    /// Interpret logical values as unix timestamps and format in UTC.
    UtcDateTime { show_seconds: bool },
    /// Select UTC format detail based on current visible span (zoom level).
    UtcAdaptive,
}

impl Default for TimeAxisLabelPolicy {
    fn default() -> Self {
        Self::LogicalDecimal { precision: 2 }
    }
}

/// Timezone alignment used by UTC-based time-axis policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TimeAxisTimeZone {
    #[default]
    Utc,
    FixedOffsetMinutes {
        minutes: i16,
    },
}

impl TimeAxisTimeZone {
    #[must_use]
    pub(super) fn offset_minutes(self) -> i16 {
        match self {
            Self::Utc => 0,
            Self::FixedOffsetMinutes { minutes } => minutes,
        }
    }

    #[must_use]
    pub(super) fn fixed_offset(self) -> FixedOffset {
        let seconds = i32::from(self.offset_minutes()) * 60;
        FixedOffset::east_opt(seconds)
            .unwrap_or_else(|| FixedOffset::east_opt(0).expect("zero UTC offset is valid"))
    }
}

/// Optional trading-session envelope used by time-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeAxisSessionConfig {
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
}

impl TimeAxisSessionConfig {
    #[must_use]
    pub(super) fn start_minute_of_day(self) -> u16 {
        u16::from(self.start_hour) * 60 + u16::from(self.start_minute)
    }

    #[must_use]
    pub(super) fn end_minute_of_day(self) -> u16 {
        u16::from(self.end_hour) * 60 + u16::from(self.end_minute)
    }

    #[must_use]
    pub(super) fn contains_local_minute(self, minute_of_day: u16) -> bool {
        let start = self.start_minute_of_day();
        let end = self.end_minute_of_day();
        if start < end {
            minute_of_day >= start && minute_of_day <= end
        } else {
            minute_of_day >= start || minute_of_day <= end
        }
    }

    #[must_use]
    pub(super) fn is_boundary(self, minute_of_day: u16, second: u32) -> bool {
        if second != 0 {
            return false;
        }
        minute_of_day == self.start_minute_of_day() || minute_of_day == self.end_minute_of_day()
    }
}

/// Runtime formatter configuration for the time axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TimeAxisLabelConfig {
    pub locale: AxisLabelLocale,
    pub policy: TimeAxisLabelPolicy,
    pub timezone: TimeAxisTimeZone,
    pub session: Option<TimeAxisSessionConfig>,
}

/// Built-in policy used for price-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PriceAxisLabelPolicy {
    /// Render price values with a fixed number of decimals.
    FixedDecimals { precision: u8 },
    /// Round prices to a deterministic minimum move before formatting.
    MinMove {
        min_move: f64,
        trim_trailing_zeros: bool,
    },
    /// Select precision from current visible price-step density.
    Adaptive,
}

impl Default for PriceAxisLabelPolicy {
    fn default() -> Self {
        Self::FixedDecimals { precision: 2 }
    }
}

/// Display transform used for price-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum PriceAxisDisplayMode {
    #[default]
    Normal,
    Percentage {
        base_price: Option<f64>,
    },
    IndexedTo100 {
        base_price: Option<f64>,
    },
}

/// Runtime formatter configuration for the price axis.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PriceAxisLabelConfig {
    pub locale: AxisLabelLocale,
    pub policy: PriceAxisLabelPolicy,
    pub display_mode: PriceAxisDisplayMode,
}
