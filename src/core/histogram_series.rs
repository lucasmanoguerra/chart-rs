use crate::core::{DataPoint, PriceScale, TimeScale, Viewport};
use crate::error::{ChartError, ChartResult};
use serde::{Deserialize, Serialize};

/// Deterministic bar geometry for histogram-style series.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HistogramBar {
    pub x_center: f64,
    pub x_left: f64,
    pub x_right: f64,
    pub y_top: f64,
    pub y_bottom: f64,
}

/// Projects point data into histogram bars.
///
/// Each bar spans from `baseline_price` to the sample value and uses fixed
/// `bar_width_px` around the mapped x-center.
pub fn project_histogram_bars(
    points: &[DataPoint],
    time_scale: TimeScale,
    price_scale: PriceScale,
    viewport: Viewport,
    bar_width_px: f64,
    baseline_price: f64,
) -> ChartResult<Vec<HistogramBar>> {
    if !bar_width_px.is_finite() || bar_width_px <= 0.0 {
        return Err(ChartError::InvalidData(
            "histogram bar width must be finite and > 0".to_owned(),
        ));
    }

    if points.is_empty() {
        return Ok(Vec::new());
    }

    let baseline_y = price_scale.price_to_pixel(baseline_price, viewport)?;
    let half_width = bar_width_px * 0.5;

    let mut bars = Vec::with_capacity(points.len());
    for point in points {
        let x_center = time_scale.time_to_pixel(point.x, viewport)?;
        let y_value = price_scale.price_to_pixel(point.y, viewport)?;
        bars.push(HistogramBar {
            x_center,
            x_left: x_center - half_width,
            x_right: x_center + half_width,
            y_top: y_value.min(baseline_y),
            y_bottom: y_value.max(baseline_y),
        });
    }

    Ok(bars)
}
