use super::axis_layout_pass_resolver::resolve_axis_layout_pass;
use super::layout_helpers::AxisLayout;

pub(super) fn resolve_price_axis_relayout_pass(
    viewport_width: f64,
    viewport_height: f64,
    requested_time_axis_height: f64,
    adaptive_price_axis_width: f64,
) -> AxisLayout {
    resolve_axis_layout_pass(
        viewport_width,
        viewport_height,
        adaptive_price_axis_width,
        requested_time_axis_height,
    )
}
