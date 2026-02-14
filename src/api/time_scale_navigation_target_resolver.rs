use crate::core::{DataPoint, OhlcBar};

pub(super) fn resolve_navigation_target_end(
    full_end: f64,
    right_offset_bars: f64,
    right_offset_px: Option<f64>,
    reference_step: Option<f64>,
    visible_span: f64,
    viewport_width: f64,
) -> f64 {
    if let Some(px) = right_offset_px {
        if viewport_width > 0.0 {
            return full_end + (visible_span.max(1e-9) / viewport_width) * px;
        }
        return full_end;
    }

    if right_offset_bars == 0.0 {
        return full_end;
    }
    match reference_step {
        Some(step) if step.is_finite() && step > 0.0 => full_end + right_offset_bars * step,
        _ => full_end,
    }
}

pub(super) fn resolve_navigation_target_range(
    full_end: f64,
    right_offset_bars: f64,
    right_offset_px: Option<f64>,
    reference_step: Option<f64>,
    visible_span: f64,
    viewport_width: f64,
) -> (f64, f64) {
    let target_end = resolve_navigation_target_end(
        full_end,
        right_offset_bars,
        right_offset_px,
        reference_step,
        visible_span,
        viewport_width,
    );
    let target_start = target_end - visible_span.max(1e-9);
    (target_start, target_end)
}

pub(super) fn resolve_reference_time_step(
    points: &[DataPoint],
    candles: &[OhlcBar],
) -> Option<f64> {
    if let Some(step) = estimate_positive_time_step(candles.iter().map(|bar| bar.time)) {
        return Some(step);
    }
    estimate_positive_time_step(points.iter().map(|point| point.x))
}

fn estimate_positive_time_step<I>(times: I) -> Option<f64>
where
    I: IntoIterator<Item = f64>,
{
    let mut ordered = times
        .into_iter()
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    if ordered.len() < 2 {
        return None;
    }

    ordered.sort_by(|left, right| left.total_cmp(right));

    let mut deltas = Vec::with_capacity(ordered.len().saturating_sub(1));
    for window in ordered.windows(2) {
        let delta = window[1] - window[0];
        if delta.is_finite() && delta > 0.0 {
            deltas.push(delta);
        }
    }

    if !deltas.is_empty() {
        deltas.sort_by(|left, right| left.total_cmp(right));
        let mid = deltas.len() / 2;
        if deltas.len() % 2 == 1 {
            return Some(deltas[mid]);
        }
        return Some((deltas[mid - 1] + deltas[mid]) * 0.5);
    }

    let span = ordered.last().copied().unwrap_or(0.0) - ordered.first().copied().unwrap_or(0.0);
    if span > 0.0 {
        let count = ordered.len().saturating_sub(1) as f64;
        return Some(span / count.max(1.0));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_navigation_target_end, resolve_navigation_target_range, resolve_reference_time_step,
    };
    use crate::core::{DataPoint, OhlcBar};

    #[test]
    fn navigation_target_prefers_right_offset_px_when_present() {
        let target = resolve_navigation_target_end(100.0, 5.0, Some(40.0), Some(2.0), 20.0, 200.0);
        assert!((target - 104.0).abs() <= 1e-9);
    }

    #[test]
    fn navigation_target_uses_bar_offset_without_px_policy() {
        let target = resolve_navigation_target_end(100.0, 3.0, None, Some(2.0), 20.0, 200.0);
        assert!((target - 106.0).abs() <= 1e-9);
    }

    #[test]
    fn reference_step_prefers_candles_then_points() {
        let candles = vec![
            OhlcBar::new(10.0, 1.0, 2.0, 1.0, 2.0).expect("c1"),
            OhlcBar::new(20.0, 1.0, 2.0, 1.0, 2.0).expect("c2"),
            OhlcBar::new(30.0, 1.0, 2.0, 1.0, 2.0).expect("c3"),
        ];
        let points = vec![DataPoint::new(10.0, 1.0), DataPoint::new(14.0, 1.0)];

        let step = resolve_reference_time_step(&points, &candles).expect("step");
        assert!((step - 10.0).abs() <= 1e-9);
    }

    #[test]
    fn navigation_target_range_uses_end_and_span() {
        let (start, end) =
            resolve_navigation_target_range(100.0, 2.0, None, Some(5.0), 20.0, 200.0);
        assert!((end - 110.0).abs() <= 1e-9);
        assert!((start - 90.0).abs() <= 1e-9);
    }
}
