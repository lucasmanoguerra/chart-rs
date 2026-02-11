use crate::core::{DataPoint, OhlcBar};

/// Returns points whose logical time falls inside an inclusive time window.
#[must_use]
pub fn points_in_time_window(points: &[DataPoint], start: f64, end: f64) -> Vec<DataPoint> {
    let (min_t, max_t) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    points
        .iter()
        .copied()
        .filter(|point| point.x >= min_t && point.x <= max_t)
        .collect()
}

/// Returns candles whose logical time falls inside an inclusive time window.
#[must_use]
pub fn candles_in_time_window(candles: &[OhlcBar], start: f64, end: f64) -> Vec<OhlcBar> {
    let (min_t, max_t) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    candles
        .iter()
        .copied()
        .filter(|candle| candle.time >= min_t && candle.time <= max_t)
        .collect()
}
