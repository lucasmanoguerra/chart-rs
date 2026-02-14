use super::layout_helpers::{AxisLayout, resolve_axis_layout};

pub(super) fn resolve_axis_layout_pass(
    viewport_width: f64,
    viewport_height: f64,
    requested_price_axis_width: f64,
    requested_time_axis_height: f64,
) -> AxisLayout {
    resolve_axis_layout(
        viewport_width,
        viewport_height,
        requested_price_axis_width,
        requested_time_axis_height,
    )
}
