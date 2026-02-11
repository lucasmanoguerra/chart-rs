use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::core::primitives::{datetime_to_unix_seconds, decimal_to_f64};
use crate::error::ChartResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub fn is_valid(self) -> bool {
        self.width > 0 && self.height > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

impl DataPoint {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn from_decimal_time(time: DateTime<Utc>, price: Decimal) -> ChartResult<Self> {
        Ok(Self {
            x: datetime_to_unix_seconds(time),
            y: decimal_to_f64(price, "price")?,
        })
    }
}
