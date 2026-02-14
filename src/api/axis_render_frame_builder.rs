use crate::core::PaneId;
use crate::error::ChartResult;
use crate::render::{
    CanvasLayerKind, LayeredRenderFrame, LinePrimitive, RectPrimitive, RenderFrame, Renderer,
    TextPrimitive,
};

use super::axis_price_scene_builder::AxisPriceSceneContext;
use super::axis_ticks::{
    AXIS_PRICE_MIN_SPACING_PX, AXIS_PRICE_TARGET_SPACING_PX, AXIS_TIME_MIN_SPACING_PX,
    AXIS_TIME_TARGET_SPACING_PX, axis_tick_target_count_with_density,
};
use super::axis_time_scene_builder::AxisTimeSceneContext;
use super::{ChartEngine, RenderStyle};

#[derive(Debug, Clone, Copy)]
pub(super) struct AxisRenderContext {
    pub main_pane_id: PaneId,
    pub plot_right: f64,
    pub plot_bottom: f64,
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub visible_start: f64,
    pub visible_end: f64,
    pub visible_span_abs: f64,
    pub style: RenderStyle,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AxisPriceDisplayContext {
    pub fallback_display_base_price: f64,
    pub display_tick_step_abs: f64,
    pub display_suffix: &'static str,
}

pub(super) struct AxisPrimitiveSink<'a> {
    frame: &'a mut RenderFrame,
    layered: &'a mut LayeredRenderFrame,
    pane_id: PaneId,
}

impl<'a> AxisPrimitiveSink<'a> {
    pub(super) fn new(
        frame: &'a mut RenderFrame,
        layered: &'a mut LayeredRenderFrame,
        pane_id: PaneId,
    ) -> Self {
        Self {
            frame,
            layered,
            pane_id,
        }
    }

    pub(super) fn push_line(&mut self, layer: CanvasLayerKind, line: LinePrimitive) {
        self.frame.lines.push(line);
        let idx = self.frame.lines.len() - 1;
        self.layered
            .push_line(self.pane_id, layer, self.frame.lines[idx]);
    }

    pub(super) fn push_rect(&mut self, layer: CanvasLayerKind, rect: RectPrimitive) {
        self.frame.rects.push(rect);
        let idx = self.frame.rects.len() - 1;
        self.layered
            .push_rect(self.pane_id, layer, self.frame.rects[idx]);
    }

    pub(super) fn push_text(&mut self, layer: CanvasLayerKind, text: TextPrimitive) {
        self.frame.texts.push(text);
        let idx = self.frame.texts.len() - 1;
        self.layered
            .push_text(self.pane_id, layer, self.frame.texts[idx].clone());
    }
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_axis_primitives(
        &self,
        frame: &mut RenderFrame,
        layered: &mut LayeredRenderFrame,
        ctx: AxisRenderContext,
    ) -> ChartResult<AxisPriceDisplayContext> {
        let main_pane_id = ctx.main_pane_id;
        let plot_right = ctx.plot_right;
        let plot_bottom = ctx.plot_bottom;
        let viewport_width = ctx.viewport_width;
        let viewport_height = ctx.viewport_height;
        let visible_start = ctx.visible_start;
        let visible_end = ctx.visible_end;
        let visible_span_abs = ctx.visible_span_abs;
        let style = ctx.style;

        let time_density_scale = self.resolve_time_axis_density_scale();
        let price_density_scale = self.resolve_price_axis_density_scale();
        let price_axis_span_px = self.resolve_price_axis_span_px(plot_bottom)?;
        let time_tick_count = axis_tick_target_count_with_density(
            plot_right,
            AXIS_TIME_TARGET_SPACING_PX,
            AXIS_TIME_MIN_SPACING_PX,
            2,
            12,
            time_density_scale,
        );
        let price_tick_count = axis_tick_target_count_with_density(
            price_axis_span_px,
            AXIS_PRICE_TARGET_SPACING_PX,
            AXIS_PRICE_MIN_SPACING_PX,
            2,
            16,
            price_density_scale,
        );

        let mut sink = AxisPrimitiveSink::new(frame, layered, main_pane_id);

        // Axis borders remain explicit frame primitives, keeping visual output
        // deterministic across all renderer backends.
        if style.show_time_axis_border {
            sink.push_line(
                CanvasLayerKind::Axis,
                LinePrimitive::new(
                    0.0,
                    plot_bottom,
                    viewport_width,
                    plot_bottom,
                    style.axis_line_width,
                    style.axis_border_color,
                ),
            );
        }
        if style.show_price_axis_border {
            sink.push_line(
                CanvasLayerKind::Axis,
                LinePrimitive::new(
                    plot_right,
                    0.0,
                    plot_right,
                    viewport_height,
                    style.axis_line_width,
                    style.axis_border_color,
                ),
            );
        }

        self.append_time_axis_scene(
            &mut sink,
            AxisTimeSceneContext {
                plot_right,
                plot_bottom,
                viewport_height,
                visible_span_abs,
                time_tick_count,
                style,
            },
        )?;

        self.append_price_axis_scene(
            &mut sink,
            AxisPriceSceneContext {
                plot_right,
                plot_bottom,
                viewport_width,
                visible_start,
                visible_end,
                price_tick_count,
                style,
            },
        )
    }
}
