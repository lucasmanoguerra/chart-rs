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

pub(super) fn axis_tick_target_count_with_density(
    axis_span_px: f64,
    target_spacing_px: f64,
    min_spacing_px: f64,
    min_ticks: usize,
    max_ticks: usize,
    density_scale: f64,
) -> usize {
    let base = axis_tick_target_count(axis_span_px, target_spacing_px, min_ticks, max_ticks);
    if !density_scale.is_finite() || density_scale <= 0.0 {
        return base;
    }

    let effective_spacing_scale = density_scale.clamp(0.50, 1.90);
    let effective_spacing = (target_spacing_px * effective_spacing_scale).max(min_spacing_px);
    let density_max_ticks = ((max_ticks as f64) / effective_spacing_scale)
        .round()
        .clamp(min_ticks as f64, (max_ticks.saturating_mul(3)) as f64)
        as usize;
    let spacing_cap_ticks =
        if axis_span_px.is_finite() && axis_span_px > 0.0 && min_spacing_px > 0.0 {
            (axis_span_px / min_spacing_px).floor() as usize
        } else {
            max_ticks.saturating_mul(3)
        };
    let effective_max_ticks = density_max_ticks.min(spacing_cap_ticks.max(min_ticks));

    axis_tick_target_count(
        axis_span_px,
        effective_spacing,
        min_ticks,
        effective_max_ticks,
    )
}

pub(super) fn density_scale_from_zoom_ratio(
    zoom_ratio: f64,
    neutral_band: f64,
    zoom_in_exponent: f64,
    zoom_out_exponent: f64,
    min_scale: f64,
    max_scale: f64,
) -> f64 {
    if !zoom_ratio.is_finite() || zoom_ratio <= 0.0 {
        return 1.0;
    }

    let neutral_band = if neutral_band.is_finite() {
        neutral_band.max(0.0)
    } else {
        0.0
    };
    if (zoom_ratio - 1.0).abs() <= neutral_band {
        return 1.0;
    }

    let zoom_in_exponent = if zoom_in_exponent.is_finite() && zoom_in_exponent > 0.0 {
        zoom_in_exponent
    } else {
        1.0
    };
    let zoom_out_exponent = if zoom_out_exponent.is_finite() && zoom_out_exponent > 0.0 {
        zoom_out_exponent
    } else {
        1.0
    };
    let exponent = if zoom_ratio < 1.0 {
        zoom_in_exponent
    } else {
        zoom_out_exponent
    };

    let min_scale = if min_scale.is_finite() {
        min_scale.max(0.01)
    } else {
        0.01
    };
    let max_scale = if max_scale.is_finite() {
        max_scale.max(min_scale)
    } else {
        min_scale
    };

    zoom_ratio.powf(exponent).clamp(min_scale, max_scale)
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

pub(super) fn select_positions_with_min_spacing_prioritized<T: Copy>(
    mut items: Vec<(T, f64, bool)>,
    min_spacing_px: f64,
) -> Vec<(T, f64, bool)> {
    if items.is_empty() {
        return items;
    }

    items.sort_by(|left, right| left.1.total_cmp(&right.1));
    if items.len() == 1 || !min_spacing_px.is_finite() || min_spacing_px <= 0.0 {
        return items;
    }

    let mut selected: Vec<(T, f64, bool)> = Vec::with_capacity(items.len());
    selected.push(items[0]);

    for item in items.iter().copied().skip(1) {
        let selected_len = selected.len();
        let last = selected
            .last()
            .copied()
            .expect("selected has at least one item");
        if item.1 - last.1 >= min_spacing_px {
            selected.push(item);
            continue;
        }

        // When candidates collide, prioritize keeping major markers.
        if item.2 && !last.2 {
            let can_replace_last =
                selected_len <= 1 || item.1 - selected[selected_len - 2].1 >= min_spacing_px;
            if can_replace_last {
                let last_index = selected_len - 1;
                selected[last_index] = item;
            }
        }
    }

    let last_item = items[items.len() - 1];
    let selected_last = selected[selected.len() - 1];
    let tail_already_selected = (selected_last.1 - last_item.1).abs() <= 1e-9;
    if !tail_already_selected {
        if selected.len() == 1 {
            if last_item.2 || !selected[0].2 {
                selected[0] = last_item;
            }
        } else {
            let penultimate = selected[selected.len() - 2];
            if last_item.1 - penultimate.1 >= min_spacing_px {
                let last_index = selected.len() - 1;
                if !selected[last_index].2 || last_item.2 {
                    selected[last_index] = last_item;
                }
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

#[cfg(test)]
mod tests {
    use super::{density_scale_from_zoom_ratio, select_positions_with_min_spacing_prioritized};

    #[test]
    fn density_scale_is_one_inside_neutral_band() {
        let scale = density_scale_from_zoom_ratio(1.04, 0.05, 0.70, 0.62, 0.45, 1.90);
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn density_scale_is_monotonic_across_zoom_in_and_zoom_out_paths() {
        let deep_in = density_scale_from_zoom_ratio(0.08, 0.05, 0.70, 0.62, 0.45, 1.90);
        let mid_in = density_scale_from_zoom_ratio(0.40, 0.05, 0.70, 0.62, 0.45, 1.90);
        let mid_out = density_scale_from_zoom_ratio(1.40, 0.05, 0.70, 0.62, 0.45, 1.90);
        let deep_out = density_scale_from_zoom_ratio(8.00, 0.05, 0.70, 0.62, 0.45, 1.90);

        assert!(deep_in < mid_in);
        assert!(mid_in < 1.0);
        assert!(1.0 < mid_out);
        assert!(mid_out < deep_out);
    }

    #[test]
    fn density_scale_handles_non_finite_inputs_with_safe_fallback() {
        let scale = density_scale_from_zoom_ratio(f64::NAN, 0.05, 0.70, 0.62, 0.45, 1.90);
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn prioritized_selector_keeps_major_when_minor_collides() {
        let selected = select_positions_with_min_spacing_prioritized(
            vec![
                (1u8, 0.0, false),
                (2u8, 30.0, false),
                (3u8, 40.0, true),
                (4u8, 100.0, false),
            ],
            56.0,
        );
        let ids: Vec<u8> = selected.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, vec![3, 4]);
    }

    #[test]
    fn prioritized_selector_preserves_spacing_and_tail_major() {
        let selected = select_positions_with_min_spacing_prioritized(
            vec![
                (1u8, 0.0, false),
                (2u8, 60.0, false),
                (3u8, 120.0, false),
                (4u8, 170.0, true),
            ],
            56.0,
        );
        let ids: Vec<u8> = selected.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, vec![1, 2, 4]);
    }
}
