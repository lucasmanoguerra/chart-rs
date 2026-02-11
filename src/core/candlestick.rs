use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "parallel-projection")]
use rayon::prelude::*;

use crate::core::primitives::{datetime_to_unix_seconds, decimal_to_f64};
use crate::core::{PriceScale, TimeScale, Viewport};
use crate::error::{ChartError, ChartResult};

/// Canonical OHLC candle used by candlestick series.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OhlcBar {
    pub time: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl OhlcBar {
    /// Builds a validated OHLC bar from raw floating values.
    ///
    /// Invariants:
    /// - all values are finite
    /// - `low <= high`
    /// - `open` and `close` are within `[low, high]`
    pub fn new(time: f64, open: f64, high: f64, low: f64, close: f64) -> ChartResult<Self> {
        if !time.is_finite()
            || !open.is_finite()
            || !high.is_finite()
            || !low.is_finite()
            || !close.is_finite()
        {
            return Err(ChartError::InvalidData(
                "ohlc values must be finite".to_owned(),
            ));
        }

        if low > high {
            return Err(ChartError::InvalidData(
                "ohlc low must be <= high".to_owned(),
            ));
        }

        if open < low || open > high || close < low || close > high {
            return Err(ChartError::InvalidData(
                "ohlc open/close must be within low/high range".to_owned(),
            ));
        }

        Ok(Self {
            time,
            open,
            high,
            low,
            close,
        })
    }

    /// Converts strongly-typed temporal/decimal input into a validated OHLC bar.
    pub fn from_decimal_time(
        time: DateTime<Utc>,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
    ) -> ChartResult<Self> {
        Self::new(
            datetime_to_unix_seconds(time),
            decimal_to_f64(open, "open")?,
            decimal_to_f64(high, "high")?,
            decimal_to_f64(low, "low")?,
            decimal_to_f64(close, "close")?,
        )
    }

    /// Returns `true` when close price is greater than or equal to open price.
    #[must_use]
    pub fn is_bullish(self) -> bool {
        self.close >= self.open
    }
}

/// Projected candle geometry in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CandleGeometry {
    pub center_x: f64,
    pub body_left: f64,
    pub body_right: f64,
    pub body_top: f64,
    pub body_bottom: f64,
    pub wick_top: f64,
    pub wick_bottom: f64,
    pub is_bullish: bool,
}

/// Projects OHLC candles into deterministic render geometry.
///
/// The function is intentionally pure and side-effect free so it can be used
/// both in rendering and in regression tests.
pub fn project_candles(
    bars: &[OhlcBar],
    time_scale: TimeScale,
    price_scale: PriceScale,
    viewport: Viewport,
    body_width_px: f64,
) -> ChartResult<Vec<CandleGeometry>> {
    if !body_width_px.is_finite() || body_width_px <= 0.0 {
        return Err(ChartError::InvalidData(
            "body width must be finite and > 0".to_owned(),
        ));
    }

    // For large series, optional parallel projection keeps API behavior stable
    // while reducing wall-clock projection time.
    #[cfg(feature = "parallel-projection")]
    {
        let projected: Vec<ChartResult<CandleGeometry>> = bars
            .par_iter()
            .map(|bar| {
                project_single_candle(*bar, time_scale, price_scale, viewport, body_width_px)
            })
            .collect();
        projected.into_iter().collect()
    }

    #[cfg(not(feature = "parallel-projection"))]
    {
        let mut out = Vec::with_capacity(bars.len());
        for bar in bars {
            out.push(project_single_candle(
                *bar,
                time_scale,
                price_scale,
                viewport,
                body_width_px,
            )?);
        }
        Ok(out)
    }
}

fn project_single_candle(
    bar: OhlcBar,
    time_scale: TimeScale,
    price_scale: PriceScale,
    viewport: Viewport,
    body_width_px: f64,
) -> ChartResult<CandleGeometry> {
    let half = body_width_px / 2.0;
    let center_x = time_scale.time_to_pixel(bar.time, viewport)?;
    let open_y = price_scale.price_to_pixel(bar.open, viewport)?;
    let close_y = price_scale.price_to_pixel(bar.close, viewport)?;
    let wick_top = price_scale.price_to_pixel(bar.high, viewport)?;
    let wick_bottom = price_scale.price_to_pixel(bar.low, viewport)?;

    Ok(CandleGeometry {
        center_x,
        body_left: center_x - half,
        body_right: center_x + half,
        body_top: open_y.min(close_y),
        body_bottom: open_y.max(close_y),
        wick_top,
        wick_bottom,
        is_bullish: bar.is_bullish(),
    })
}
