use super::axis_price_width_tick_context_resolver::PriceAxisWidthTickContext;
use super::axis_render_frame_builder::AxisPriceDisplayContext;

pub(super) fn resolve_price_axis_width_display_context(
    width_tick_context: &PriceAxisWidthTickContext,
) -> AxisPriceDisplayContext {
    width_tick_context.display_context
}
