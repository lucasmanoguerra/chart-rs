use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::core::{CandleGeometry, DataPoint, Viewport};
use crate::interaction::CrosshairState;

use super::{PriceLabelCacheStats, TimeLabelCacheStats};

/// Serializable deterministic state snapshot used by regression tests and
/// debugging tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrosshairFormatterOverrideMode {
    None,
    Legacy,
    Context,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosshairFormatterSnapshot {
    pub time_override_mode: CrosshairFormatterOverrideMode,
    pub price_override_mode: CrosshairFormatterOverrideMode,
    pub time_formatter_generation: u64,
    pub price_formatter_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosshairFormatterDiagnostics {
    pub time_override_mode: CrosshairFormatterOverrideMode,
    pub price_override_mode: CrosshairFormatterOverrideMode,
    pub time_formatter_generation: u64,
    pub price_formatter_generation: u64,
    pub time_cache: TimeLabelCacheStats,
    pub price_cache: PriceLabelCacheStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub viewport: Viewport,
    pub time_full_range: (f64, f64),
    pub time_visible_range: (f64, f64),
    pub price_domain: (f64, f64),
    pub crosshair: CrosshairState,
    pub points: Vec<DataPoint>,
    pub candle_geometry: Vec<CandleGeometry>,
    pub series_metadata: IndexMap<String, String>,
    pub crosshair_formatter: CrosshairFormatterSnapshot,
}
