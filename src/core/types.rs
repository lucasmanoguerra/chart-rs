use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::primitives::{datetime_to_unix_seconds, decimal_to_f64};
use crate::error::ChartResult;

/// Pixel dimensions for a render target.
///
/// `Viewport` is intentionally copyable because it is frequently read by
/// mapping/projection functions in hot paths.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    /// Constructs a viewport with raw dimensions in pixels.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns `true` when both dimensions are non-zero.
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Minimal XY sample used by line/area-like data paths.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

impl DataPoint {
    /// Creates a data point from floating values.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Creates a data point from strongly-typed temporal and decimal values.
    ///
    /// This constructor is useful when upstream market data is already typed
    /// and we want a single checked conversion boundary into chart internals.
    pub fn from_decimal_time(time: DateTime<Utc>, price: Decimal) -> ChartResult<Self> {
        Ok(Self {
            x: datetime_to_unix_seconds(time),
            y: decimal_to_f64(price, "price")?,
        })
    }
}
