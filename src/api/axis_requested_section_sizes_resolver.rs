use super::RenderStyle;
use super::axis_time_axis_height_estimator::estimate_required_time_axis_height;

#[derive(Debug, Clone, Copy)]
pub(super) struct RequestedAxisSectionSizes {
    pub requested_price_axis_width: f64,
    pub requested_time_axis_height: f64,
}

pub(super) fn resolve_requested_axis_section_sizes(
    style: RenderStyle,
) -> RequestedAxisSectionSizes {
    let requested_price_axis_width = style.price_axis_width_px;
    let requested_time_axis_height = style
        .time_axis_height_px
        .max(estimate_required_time_axis_height(style));

    RequestedAxisSectionSizes {
        requested_price_axis_width,
        requested_time_axis_height,
    }
}
