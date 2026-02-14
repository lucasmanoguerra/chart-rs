use super::RenderStyle;

#[derive(Debug, Clone, Copy)]
pub(super) struct AxisPriceSceneLayout {
    pub price_axis_label_anchor_x: f64,
    pub last_price_label_anchor_x: f64,
    pub price_axis_tick_mark_end_x: f64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AxisPriceSceneLayoutContext {
    pub plot_right: f64,
    pub viewport_width: f64,
    pub style: RenderStyle,
}

pub(super) fn build_axis_price_scene_layout(
    ctx: AxisPriceSceneLayoutContext,
) -> AxisPriceSceneLayout {
    let plot_right = ctx.plot_right;
    let viewport_width = ctx.viewport_width;
    let style = ctx.style;

    let price_axis_label_anchor_x = (viewport_width - style.price_axis_label_padding_right_px)
        .clamp(plot_right, viewport_width);
    let last_price_label_anchor_x = (viewport_width - style.last_price_label_padding_right_px)
        .clamp(plot_right, viewport_width);
    let price_axis_tick_mark_end_x =
        (plot_right + style.price_axis_tick_mark_length_px).clamp(plot_right, viewport_width);

    AxisPriceSceneLayout {
        price_axis_label_anchor_x,
        last_price_label_anchor_x,
        price_axis_tick_mark_end_x,
    }
}
