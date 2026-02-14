use crate::error::ChartResult;
use crate::render::{CanvasLayerKind, Renderer, TextHAlign, TextPrimitive};

use super::axis_label_format::is_major_time_tick;
use super::axis_render_frame_builder::AxisPrimitiveSink;
use super::axis_ticks::{
    AXIS_TIME_MIN_SPACING_PX, axis_ticks, select_positions_with_min_spacing_prioritized,
    tick_step_hint_from_values,
};
use super::layout_helpers::estimate_label_text_width_px;
use super::{ChartEngine, RenderStyle};

#[derive(Debug, Clone, Copy)]
pub(super) struct AxisTimeSceneContext {
    pub plot_right: f64,
    pub plot_bottom: f64,
    pub viewport_height: f64,
    pub visible_span_abs: f64,
    pub time_tick_count: usize,
    pub style: RenderStyle,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_time_axis_scene(
        &self,
        sink: &mut AxisPrimitiveSink<'_>,
        ctx: AxisTimeSceneContext,
    ) -> ChartResult<()> {
        let plot_right = ctx.plot_right;
        let plot_bottom = ctx.plot_bottom;
        let viewport_height = ctx.viewport_height;
        let visible_span_abs = ctx.visible_span_abs;
        let time_tick_count = ctx.time_tick_count;
        let style = ctx.style;

        let raw_time_ticks =
            axis_ticks(self.core.model.time_scale.visible_range(), time_tick_count);
        let time_tick_step_abs = tick_step_hint_from_values(&raw_time_ticks).abs();
        let mut time_label_min_spacing_px = AXIS_TIME_MIN_SPACING_PX;
        if style.show_time_axis_labels {
            let mut max_label_width_px: f64 = 0.0;
            for time in raw_time_ticks.iter().copied() {
                let is_major_tick =
                    is_major_time_tick(time, self.core.behavior.time_axis_label_config);
                let label_font_size_px = if is_major_tick {
                    style.major_time_label_font_size_px
                } else {
                    style.time_axis_label_font_size_px
                };
                let text = self.format_time_axis_tick_label(
                    time,
                    visible_span_abs,
                    time_tick_step_abs,
                    is_major_tick,
                );
                let measured_width = estimate_label_text_width_px(&text, label_font_size_px);
                let capped_width =
                    measured_width.min(Self::lwc_time_label_width_budget_px(label_font_size_px));
                max_label_width_px = max_label_width_px.max(capped_width);
            }
            if max_label_width_px.is_finite() && max_label_width_px > 0.0 {
                time_label_min_spacing_px = time_label_min_spacing_px
                    .max((max_label_width_px + 4.0).min(plot_right.max(AXIS_TIME_MIN_SPACING_PX)));
            }
        }

        let mut time_ticks = Vec::with_capacity(time_tick_count);
        for time in raw_time_ticks {
            let px = self
                .core
                .model
                .time_scale
                .time_to_pixel(time, self.core.model.viewport)?;
            let clamped_px = px.clamp(0.0, plot_right);
            let is_major_tick = is_major_time_tick(time, self.core.behavior.time_axis_label_config);
            time_ticks.push((time, clamped_px, is_major_tick));
        }

        let mut time_label_candidates: Vec<(TextPrimitive, bool)> = Vec::new();
        for (time, px, is_major_tick) in
            select_positions_with_min_spacing_prioritized(time_ticks, time_label_min_spacing_px)
        {
            let (
                grid_color,
                grid_line_width,
                label_font_size_px,
                label_offset_y_px,
                label_color,
                tick_mark_color,
                tick_mark_width,
                tick_mark_length_px,
            ) = if is_major_tick {
                (
                    style.major_grid_line_color,
                    style.major_grid_line_width,
                    style.major_time_label_font_size_px,
                    style.major_time_label_offset_y_px,
                    style.major_time_label_color,
                    style.major_time_tick_mark_color,
                    style.major_time_tick_mark_width,
                    style.major_time_tick_mark_length_px,
                )
            } else {
                (
                    style.grid_line_color,
                    style.grid_line_width,
                    style.time_axis_label_font_size_px,
                    style.time_axis_label_offset_y_px,
                    style.time_axis_label_color,
                    style.time_axis_tick_mark_color,
                    style.time_axis_tick_mark_width,
                    style.time_axis_tick_mark_length_px,
                )
            };
            let time_label_y = (plot_bottom + label_offset_y_px)
                .min((viewport_height - label_font_size_px).max(0.0));
            let text = self.format_time_axis_tick_label(
                time,
                visible_span_abs,
                time_tick_step_abs,
                is_major_tick,
            );
            if style.show_time_axis_labels && (!is_major_tick || style.show_major_time_labels) {
                let estimated_width = estimate_label_text_width_px(&text, label_font_size_px);
                if estimated_width <= (plot_right - 2.0).max(0.0) {
                    let half_width = (estimated_width * 0.5).clamp(0.0, plot_right * 0.5);
                    let time_label_x =
                        px.clamp(half_width, (plot_right - half_width).max(half_width));
                    time_label_candidates.push((
                        TextPrimitive::new(
                            text,
                            time_label_x,
                            time_label_y,
                            label_font_size_px,
                            label_color,
                            TextHAlign::Center,
                        ),
                        is_major_tick,
                    ));
                }
            }
            if !is_major_tick || style.show_major_time_grid_lines {
                sink.push_line(
                    CanvasLayerKind::Grid,
                    crate::render::LinePrimitive::new(
                        px,
                        0.0,
                        px,
                        plot_bottom,
                        grid_line_width,
                        grid_color,
                    ),
                );
            }
            if style.show_time_axis_tick_marks
                && (!is_major_tick || style.show_major_time_tick_marks)
            {
                sink.push_line(
                    CanvasLayerKind::Axis,
                    crate::render::LinePrimitive::new(
                        px,
                        plot_bottom,
                        px,
                        (plot_bottom + tick_mark_length_px).min(viewport_height),
                        tick_mark_width,
                        tick_mark_color,
                    ),
                );
            }
        }

        if !time_label_candidates.is_empty() {
            let index_candidates: Vec<(usize, f64, bool)> = time_label_candidates
                .iter()
                .enumerate()
                .map(|(index, (label, is_major))| (index, label.x, *is_major))
                .collect();
            let mut selected_labels: Vec<(TextPrimitive, bool)> =
                select_positions_with_min_spacing_prioritized(
                    index_candidates,
                    time_label_min_spacing_px,
                )
                .into_iter()
                .map(|(index, _, _)| time_label_candidates[index].clone())
                .collect();
            selected_labels.sort_by(|left, right| left.0.x.total_cmp(&right.0.x));

            if selected_labels.len() >= 3 {
                let first_gap = selected_labels[1].0.x - selected_labels[0].0.x;
                let second_gap = selected_labels[2].0.x - selected_labels[1].0.x;
                if first_gap > second_gap * 1.70 && !selected_labels[0].1 {
                    selected_labels.remove(0);
                }
            }
            if selected_labels.len() >= 3 {
                let len = selected_labels.len();
                let last_gap = selected_labels[len - 1].0.x - selected_labels[len - 2].0.x;
                let penultimate_gap = selected_labels[len - 2].0.x - selected_labels[len - 3].0.x;
                if last_gap > penultimate_gap * 1.70 && !selected_labels[len - 1].1 {
                    selected_labels.pop();
                }
            }

            for (label, _) in selected_labels {
                sink.push_text(CanvasLayerKind::Axis, label);
            }
        }

        Ok(())
    }
}
