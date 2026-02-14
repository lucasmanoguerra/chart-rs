use super::RenderStyle;

pub(super) fn initialize_required_price_axis_width(style: RenderStyle) -> f64 {
    style.price_axis_width_px
}

pub(super) fn finalize_required_price_axis_width(required_width: f64) -> f64 {
    required_width.max(1.0)
}
