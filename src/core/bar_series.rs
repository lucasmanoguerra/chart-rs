use crate::core::{OhlcBar, PriceScale, TimeScale, Viewport};
use crate::error::{ChartError, ChartResult};
use serde::{Deserialize, Serialize};

/// Deterministic OHLC bar geometry in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BarGeometry {
    pub center_x: f64,
    pub high_y: f64,
    pub low_y: f64,
    pub open_y: f64,
    pub close_y: f64,
    pub open_x: f64,
    pub close_x: f64,
}

/// Projects OHLC bars into deterministic bar-series geometry.
///
/// `tick_width_px` controls the horizontal size of open/close ticks around the
/// vertical high-low stem.
pub fn project_bars(
    bars: &[OhlcBar],
    time_scale: TimeScale,
    price_scale: PriceScale,
    viewport: Viewport,
    tick_width_px: f64,
) -> ChartResult<Vec<BarGeometry>> {
    if !tick_width_px.is_finite() || tick_width_px <= 0.0 {
        return Err(ChartError::InvalidData(
            "tick width must be finite and > 0".to_owned(),
        ));
    }

    let half = tick_width_px * 0.5;
    let mut projected = Vec::with_capacity(bars.len());
    for bar in bars {
        let center_x = time_scale.time_to_pixel(bar.time, viewport)?;
        let open_y = price_scale.price_to_pixel(bar.open, viewport)?;
        let close_y = price_scale.price_to_pixel(bar.close, viewport)?;
        let high_y = price_scale.price_to_pixel(bar.high, viewport)?;
        let low_y = price_scale.price_to_pixel(bar.low, viewport)?;

        projected.push(BarGeometry {
            center_x,
            high_y,
            low_y,
            open_y,
            close_y,
            open_x: center_x - half,
            close_x: center_x + half,
        });
    }

    Ok(projected)
}
