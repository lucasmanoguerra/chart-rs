use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::error::{ChartError, ChartResult};

pub fn decimal_to_f64(value: Decimal, field_name: &str) -> ChartResult<f64> {
    value.to_f64().ok_or_else(|| {
        ChartError::InvalidData(format!("{field_name} cannot be represented as f64"))
    })
}

#[must_use]
pub fn datetime_to_unix_seconds(time: DateTime<Utc>) -> f64 {
    time.timestamp_millis() as f64 / 1000.0
}
