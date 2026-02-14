pub(super) fn accumulate_price_axis_width_contribution(
    required_width: f64,
    contribution_width: f64,
) -> f64 {
    required_width.max(contribution_width)
}
