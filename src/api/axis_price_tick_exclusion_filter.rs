use super::RenderStyle;
use super::last_price_axis_scene_builder::LastPriceMarker;

pub(super) fn filter_price_ticks_for_last_price_label(
    selected_price_ticks: &[(f64, f64)],
    style: RenderStyle,
    latest_price_marker: Option<LastPriceMarker>,
) -> Vec<(f64, f64)> {
    let mut ticks = selected_price_ticks.to_vec();

    if style.show_last_price_label
        && style.last_price_label_exclusion_px.is_finite()
        && style.last_price_label_exclusion_px > 0.0
    {
        if let Some(marker) = latest_price_marker {
            ticks.retain(|(_, py)| (*py - marker.py).abs() >= style.last_price_label_exclusion_px);
            if ticks.is_empty() && !selected_price_ticks.is_empty() {
                let fallback_tick = selected_price_ticks
                    .iter()
                    .copied()
                    .max_by(|left, right| {
                        (left.1 - marker.py)
                            .abs()
                            .total_cmp(&(right.1 - marker.py).abs())
                    })
                    .expect("selected price ticks not empty");
                ticks.push(fallback_tick);
            }
        }
    }

    ticks
}
