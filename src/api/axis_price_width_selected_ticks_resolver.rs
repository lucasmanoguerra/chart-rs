use super::axis_price_width_tick_context_resolver::PriceAxisWidthTickContext;

pub(super) fn resolve_price_axis_width_selected_ticks(
    width_tick_context: &PriceAxisWidthTickContext,
) -> &[(f64, f64)] {
    &width_tick_context.selected_ticks
}
