pub(super) fn accumulate_required_price_axis_width(
    required_width: f64,
    contribution_width: f64,
) -> f64 {
    required_width.max(contribution_width)
}
