pub(super) fn should_relayout_price_axis_for_adaptive_width(
    adaptive_price_axis_width: f64,
    requested_price_axis_width: f64,
) -> bool {
    adaptive_price_axis_width > requested_price_axis_width
}
