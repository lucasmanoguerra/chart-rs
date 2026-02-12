use crate::error::{ChartError, ChartResult};
use crate::extensions::SeriesMarker;

pub(super) fn expand_visible_window(range: (f64, f64), ratio: f64) -> ChartResult<(f64, f64)> {
    if !ratio.is_finite() || ratio < 0.0 {
        return Err(ChartError::InvalidData(
            "overscan ratio must be finite and >= 0".to_owned(),
        ));
    }

    let span = range.1 - range.0;
    let padding = span * ratio;
    Ok((range.0 - padding, range.1 + padding))
}

pub(super) fn markers_in_time_window(
    markers: &[SeriesMarker],
    start: f64,
    end: f64,
) -> Vec<SeriesMarker> {
    let (min_t, max_t) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    markers
        .iter()
        .filter(|marker| marker.time >= min_t && marker.time <= max_t)
        .cloned()
        .collect()
}
