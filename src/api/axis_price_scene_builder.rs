use crate::error::ChartResult;
use crate::render::Renderer;

use super::axis_price_layout_builder::{
    AxisPriceSceneLayoutContext, build_axis_price_scene_layout,
};
use super::axis_price_primitives_builder::AxisPricePrimitivesContext;
use super::axis_render_frame_builder::{AxisPriceDisplayContext, AxisPrimitiveSink};
use super::last_price_axis_scene_builder::LastPriceAxisSceneContext;
use super::{ChartEngine, RenderStyle};

#[derive(Debug, Clone, Copy)]
pub(super) struct AxisPriceSceneContext {
    pub plot_right: f64,
    pub plot_bottom: f64,
    pub viewport_width: f64,
    pub visible_start: f64,
    pub visible_end: f64,
    pub price_tick_count: usize,
    pub style: RenderStyle,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_price_axis_scene(
        &self,
        sink: &mut AxisPrimitiveSink<'_>,
        ctx: AxisPriceSceneContext,
    ) -> ChartResult<AxisPriceDisplayContext> {
        let plot_right = ctx.plot_right;
        let plot_bottom = ctx.plot_bottom;
        let viewport_width = ctx.viewport_width;
        let visible_start = ctx.visible_start;
        let visible_end = ctx.visible_end;
        let price_tick_count = ctx.price_tick_count;
        let style = ctx.style;

        let layout = build_axis_price_scene_layout(AxisPriceSceneLayoutContext {
            plot_right,
            viewport_width,
            style,
        });

        let latest_price_marker = self.resolve_last_price_marker_for_axis(
            style,
            visible_start,
            visible_end,
            plot_bottom,
        )?;
        let tick_selection = self.select_price_axis_ticks(
            price_tick_count,
            plot_bottom,
            style,
            latest_price_marker,
        )?;
        let display_ctx = self.resolve_price_axis_display_context(tick_selection.tick_step_abs);

        self.append_price_axis_tick_primitives(
            sink,
            tick_selection.ticks,
            AxisPricePrimitivesContext {
                plot_right,
                plot_bottom,
                price_axis_label_anchor_x: layout.price_axis_label_anchor_x,
                price_axis_tick_mark_end_x: layout.price_axis_tick_mark_end_x,
                fallback_display_base_price: display_ctx.fallback_display_base_price,
                display_tick_step_abs: display_ctx.display_tick_step_abs,
                display_suffix: display_ctx.display_suffix,
                style,
            },
        );

        self.append_last_price_axis_primitives(
            sink,
            latest_price_marker,
            LastPriceAxisSceneContext {
                plot_right,
                plot_bottom,
                viewport_width,
                last_price_label_anchor_x: layout.last_price_label_anchor_x,
                fallback_display_base_price: display_ctx.fallback_display_base_price,
                display_tick_step_abs: display_ctx.display_tick_step_abs,
                display_suffix: display_ctx.display_suffix,
                style,
            },
        );

        Ok(display_ctx)
    }
}
