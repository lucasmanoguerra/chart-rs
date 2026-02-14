use super::axis_ticks::{AXIS_PRICE_MIN_SPACING_PX, select_ticks_with_min_spacing};

pub(super) fn select_price_ticks_with_min_spacing(price_ticks: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    select_ticks_with_min_spacing(price_ticks, AXIS_PRICE_MIN_SPACING_PX)
}
