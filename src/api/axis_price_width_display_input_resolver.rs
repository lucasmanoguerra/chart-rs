use super::axis_price_width_display_context_resolver::resolve_price_axis_width_display_context;
use super::axis_price_width_tick_context_resolver::PriceAxisWidthTickContext;

#[derive(Debug, Clone, Copy)]
pub(super) struct PriceAxisWidthDisplayInputs {
    pub fallback_display_base_price: f64,
    pub display_tick_step_abs: f64,
    pub display_suffix: &'static str,
}

pub(super) fn resolve_price_axis_width_display_inputs(
    width_tick_context: &PriceAxisWidthTickContext,
) -> PriceAxisWidthDisplayInputs {
    let display_context = resolve_price_axis_width_display_context(width_tick_context);
    PriceAxisWidthDisplayInputs {
        fallback_display_base_price: display_context.fallback_display_base_price,
        display_tick_step_abs: display_context.display_tick_step_abs,
        display_suffix: display_context.display_suffix,
    }
}
