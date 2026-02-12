pub(super) const AXIS_TIME_TARGET_SPACING_PX: f64 = 72.0;
pub(super) const AXIS_TIME_MIN_SPACING_PX: f64 = 56.0;
pub(super) const AXIS_PRICE_TARGET_SPACING_PX: f64 = 26.0;
pub(super) const AXIS_PRICE_MIN_SPACING_PX: f64 = 22.0;

pub(super) fn axis_tick_target_count(
    axis_span_px: f64,
    target_spacing_px: f64,
    min_ticks: usize,
    max_ticks: usize,
) -> usize {
    if !axis_span_px.is_finite() || axis_span_px <= 0.0 {
        return min_ticks;
    }
    if !target_spacing_px.is_finite() || target_spacing_px <= 0.0 {
        return min_ticks;
    }

    let raw = (axis_span_px / target_spacing_px).floor() as usize + 1;
    raw.clamp(min_ticks, max_ticks)
}

pub(super) fn select_ticks_with_min_spacing(
    mut ticks: Vec<(f64, f64)>,
    min_spacing_px: f64,
) -> Vec<(f64, f64)> {
    if ticks.is_empty() {
        return ticks;
    }

    ticks.sort_by(|left, right| left.1.total_cmp(&right.1));
    if ticks.len() == 1 || !min_spacing_px.is_finite() || min_spacing_px <= 0.0 {
        return ticks;
    }

    let mut selected = Vec::with_capacity(ticks.len());
    selected.push(ticks[0]);

    for tick in ticks.iter().copied().skip(1) {
        if tick.1 - selected.last().expect("not empty").1 >= min_spacing_px {
            selected.push(tick);
        }
    }

    let last_tick = *ticks.last().expect("not empty");
    let selected_last = *selected.last().expect("not empty");
    if selected_last != last_tick {
        if selected.len() == 1 {
            // On very narrow axes a single label is clearer than overlapping pairs.
            selected[0] = last_tick;
        } else {
            let penultimate = selected[selected.len() - 2];
            if last_tick.1 - penultimate.1 >= min_spacing_px {
                let last_index = selected.len() - 1;
                selected[last_index] = last_tick;
            }
        }
    }

    selected
}

pub(super) fn axis_ticks(range: (f64, f64), tick_count: usize) -> Vec<f64> {
    if tick_count == 0 {
        return Vec::new();
    }

    if tick_count == 1 {
        return vec![range.0];
    }

    let span = range.1 - range.0;
    let denominator = (tick_count - 1) as f64;
    (0..tick_count)
        .map(|index| {
            let ratio = (index as f64) / denominator;
            range.0 + span * ratio
        })
        .collect()
}

pub(super) fn tick_step_hint_from_values(values: &[f64]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let mut best = f64::INFINITY;
    for pair in values.windows(2) {
        let step = (pair[1] - pair[0]).abs();
        if step.is_finite() && step > 0.0 {
            best = best.min(step);
        }
    }

    if best.is_finite() { best } else { 0.0 }
}
