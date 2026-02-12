use crate::core::{points_in_time_window, project_line_segments};
use crate::error::ChartResult;
use crate::render::{
    Color, LinePrimitive, RectPrimitive, RenderFrame, Renderer, TextHAlign, TextPrimitive,
};

use super::axis_label_format::{
    format_price_axis_label, format_time_axis_label, is_major_time_tick,
    map_price_step_to_display_value, map_price_to_display_value, price_display_mode_suffix,
    quantize_logical_time_millis, quantize_price_label_value,
};
use super::axis_ticks::{
    AXIS_PRICE_MIN_SPACING_PX, AXIS_PRICE_TARGET_SPACING_PX, AXIS_TIME_MIN_SPACING_PX,
    AXIS_TIME_TARGET_SPACING_PX, axis_tick_target_count, axis_ticks, select_ticks_with_min_spacing,
    tick_step_hint_from_values,
};
use super::label_cache::{PriceLabelCacheKey, TimeLabelCacheKey};
use super::layout_helpers::{
    estimate_label_text_width_px, rects_overlap, resolve_crosshair_box_vertical_layout,
    stabilize_position,
};
use super::{
    ChartEngine, CrosshairLabelBoxHorizontalAnchor, CrosshairLabelBoxOverflowPolicy,
    CrosshairLabelBoxVisibilityPriority, CrosshairLabelBoxWidthMode, CrosshairLabelBoxZOrderPolicy,
    LastPriceLabelBoxWidthMode,
};

impl<R: Renderer> ChartEngine<R> {
    fn format_time_axis_label(&self, logical_time: f64, visible_span_abs: f64) -> String {
        let profile = self.resolve_time_label_cache_profile(visible_span_abs);
        let key = TimeLabelCacheKey {
            profile,
            logical_time_millis: quantize_logical_time_millis(logical_time),
        };

        if let Some(cached) = self.time_label_cache.borrow_mut().get(key) {
            return cached;
        }

        let value = if let Some(formatter) = &self.time_label_formatter {
            formatter(logical_time)
        } else {
            format_time_axis_label(logical_time, self.time_axis_label_config, visible_span_abs)
        };
        self.time_label_cache
            .borrow_mut()
            .insert(key, value.clone());
        value
    }

    fn format_price_axis_label(
        &self,
        display_price: f64,
        tick_step_abs: f64,
        mode_suffix: &str,
    ) -> String {
        let profile = self.resolve_price_label_cache_profile();
        let key = PriceLabelCacheKey {
            profile,
            display_price_nanos: quantize_price_label_value(display_price),
            tick_step_nanos: quantize_price_label_value(tick_step_abs),
            has_percent_suffix: !mode_suffix.is_empty(),
        };

        if let Some(cached) = self.price_label_cache.borrow_mut().get(key) {
            return cached;
        }

        let mut text = if let Some(formatter) = &self.price_label_formatter {
            formatter(display_price)
        } else {
            format_price_axis_label(display_price, self.price_axis_label_config, tick_step_abs)
        };
        if !mode_suffix.is_empty() {
            text.push_str(mode_suffix);
        }
        self.price_label_cache
            .borrow_mut()
            .insert(key, text.clone());
        text
    }

    /// Materializes backend-agnostic primitives for one draw pass.
    ///
    /// This keeps geometry computation deterministic and centralized in the API
    /// layer while renderer backends only execute drawing commands.
    pub fn build_render_frame(&self) -> ChartResult<RenderFrame> {
        let mut frame = RenderFrame::new(self.viewport);
        let (visible_start, visible_end) = self.time_scale.visible_range();

        let visible_points = points_in_time_window(&self.points, visible_start, visible_end);
        let segments = project_line_segments(
            &visible_points,
            self.time_scale,
            self.price_scale,
            self.viewport,
        )?;

        let style = self.render_style;
        let series_color = style.series_line_color;
        for segment in segments {
            frame = frame.with_line(LinePrimitive::new(
                segment.x1,
                segment.y1,
                segment.x2,
                segment.y2,
                1.5,
                series_color,
            ));
        }

        let viewport_width = f64::from(self.viewport.width);
        let viewport_height = f64::from(self.viewport.height);
        let plot_right = (viewport_width - style.price_axis_width_px).clamp(0.0, viewport_width);
        let plot_bottom = (viewport_height - style.time_axis_height_px).clamp(0.0, viewport_height);
        let price_axis_label_anchor_x = (viewport_width - style.price_axis_label_padding_right_px)
            .clamp(plot_right, viewport_width);
        let last_price_label_anchor_x = (viewport_width - style.last_price_label_padding_right_px)
            .clamp(plot_right, viewport_width);
        let price_axis_tick_mark_end_x =
            (plot_right + style.price_axis_tick_mark_length_px).clamp(plot_right, viewport_width);
        let axis_color = style.axis_border_color;
        let price_label_color = style.axis_label_color;
        let time_tick_count =
            axis_tick_target_count(plot_right, AXIS_TIME_TARGET_SPACING_PX, 2, 12);
        let price_tick_count =
            axis_tick_target_count(plot_bottom, AXIS_PRICE_TARGET_SPACING_PX, 2, 16);

        // Axis borders remain explicit frame primitives, keeping visual output
        // deterministic across all renderer backends.
        if style.show_time_axis_border {
            frame = frame.with_line(LinePrimitive::new(
                0.0,
                plot_bottom,
                viewport_width,
                plot_bottom,
                style.axis_line_width,
                axis_color,
            ));
        }
        if style.show_price_axis_border {
            frame = frame.with_line(LinePrimitive::new(
                plot_right,
                0.0,
                plot_right,
                viewport_height,
                style.axis_line_width,
                axis_color,
            ));
        }

        let mut time_ticks = Vec::with_capacity(time_tick_count);
        for time in axis_ticks(self.time_scale.visible_range(), time_tick_count) {
            let px = self.time_scale.time_to_pixel(time, self.viewport)?;
            let clamped_px = px.clamp(0.0, plot_right);
            time_ticks.push((time, clamped_px));
        }

        let visible_span_abs = (visible_end - visible_start).abs();
        for (time, px) in select_ticks_with_min_spacing(time_ticks, AXIS_TIME_MIN_SPACING_PX) {
            let is_major_tick = is_major_time_tick(time, self.time_axis_label_config);
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
            let text = self.format_time_axis_label(time, visible_span_abs);
            if style.show_time_axis_labels && (!is_major_tick || style.show_major_time_labels) {
                frame = frame.with_text(TextPrimitive::new(
                    text,
                    px,
                    time_label_y,
                    label_font_size_px,
                    label_color,
                    TextHAlign::Center,
                ));
            }
            if !is_major_tick || style.show_major_time_grid_lines {
                frame = frame.with_line(LinePrimitive::new(
                    px,
                    0.0,
                    px,
                    plot_bottom,
                    grid_line_width,
                    grid_color,
                ));
            }
            if style.show_time_axis_tick_marks
                && (!is_major_tick || style.show_major_time_tick_marks)
            {
                frame = frame.with_line(LinePrimitive::new(
                    px,
                    plot_bottom,
                    px,
                    (plot_bottom + tick_mark_length_px).min(viewport_height),
                    tick_mark_width,
                    tick_mark_color,
                ));
            }
        }

        let raw_price_ticks = self.price_scale.ticks(price_tick_count)?;
        let mut price_ticks = Vec::with_capacity(raw_price_ticks.len());
        for price in raw_price_ticks.iter().copied() {
            let py = self.price_scale.price_to_pixel(price, self.viewport)?;
            let clamped_py = py.clamp(0.0, plot_bottom);
            price_ticks.push((price, clamped_py));
        }
        let price_tick_step_abs = tick_step_hint_from_values(&raw_price_ticks);
        let fallback_display_base_price = self.resolve_price_display_base_price();
        let display_tick_step_abs = map_price_step_to_display_value(
            price_tick_step_abs,
            self.price_axis_label_config.display_mode,
            fallback_display_base_price,
        )
        .abs();
        let display_suffix = price_display_mode_suffix(self.price_axis_label_config.display_mode);
        let latest_price_marker = if let Some((last_price, previous_price)) = self
            .resolve_latest_and_previous_price_values(
                style.last_price_source_mode,
                visible_start,
                visible_end,
            ) {
            let py = self
                .price_scale
                .price_to_pixel(last_price, self.viewport)?
                .clamp(0.0, plot_bottom);
            let (marker_line_color, marker_label_color) =
                self.resolve_last_price_marker_colors(last_price, previous_price);
            Some((last_price, py, marker_line_color, marker_label_color))
        } else {
            None
        };

        let selected_price_ticks =
            select_ticks_with_min_spacing(price_ticks, AXIS_PRICE_MIN_SPACING_PX);
        let mut price_ticks_for_axis = selected_price_ticks.clone();
        if style.show_last_price_label
            && style.last_price_label_exclusion_px.is_finite()
            && style.last_price_label_exclusion_px > 0.0
        {
            if let Some((_, marker_py, _, _)) = latest_price_marker {
                price_ticks_for_axis.retain(|(_, py)| {
                    (py - marker_py).abs() >= style.last_price_label_exclusion_px
                });
                if price_ticks_for_axis.is_empty() && !selected_price_ticks.is_empty() {
                    let fallback_tick = selected_price_ticks
                        .iter()
                        .copied()
                        .max_by(|left, right| {
                            (left.1 - marker_py)
                                .abs()
                                .total_cmp(&(right.1 - marker_py).abs())
                        })
                        .expect("selected price ticks not empty");
                    price_ticks_for_axis.push(fallback_tick);
                }
            }
        }

        for (price, py) in price_ticks_for_axis {
            let display_price = map_price_to_display_value(
                price,
                self.price_axis_label_config.display_mode,
                fallback_display_base_price,
            );
            let text =
                self.format_price_axis_label(display_price, display_tick_step_abs, display_suffix);
            if style.show_price_axis_labels {
                frame = frame.with_text(TextPrimitive::new(
                    text,
                    price_axis_label_anchor_x,
                    (py - style.price_axis_label_offset_y_px).max(0.0),
                    style.price_axis_label_font_size_px,
                    price_label_color,
                    TextHAlign::Right,
                ));
            }
            if style.show_price_axis_grid_lines {
                frame = frame.with_line(LinePrimitive::new(
                    0.0,
                    py,
                    plot_right,
                    py,
                    style.price_axis_grid_line_width,
                    style.price_axis_grid_line_color,
                ));
            }
            if style.show_price_axis_tick_marks {
                frame = frame.with_line(LinePrimitive::new(
                    plot_right,
                    py,
                    price_axis_tick_mark_end_x,
                    py,
                    style.price_axis_tick_mark_width,
                    style.price_axis_tick_mark_color,
                ));
            }
        }

        if let Some((last_price, py, marker_line_color, marker_label_color)) = latest_price_marker {
            if style.show_last_price_line {
                frame = frame.with_line(LinePrimitive::new(
                    0.0,
                    py,
                    plot_right,
                    py,
                    style.last_price_line_width,
                    marker_line_color,
                ));
            }

            if style.show_last_price_label {
                let display_price = map_price_to_display_value(
                    last_price,
                    self.price_axis_label_config.display_mode,
                    fallback_display_base_price,
                );
                let text = self.format_price_axis_label(
                    display_price,
                    display_tick_step_abs,
                    display_suffix,
                );
                let text_y = (py - style.last_price_label_offset_y_px).max(0.0);
                let box_fill_color =
                    self.resolve_last_price_label_box_fill_color(marker_label_color);
                let label_text_color = self
                    .resolve_last_price_label_box_text_color(box_fill_color, marker_label_color);
                let axis_panel_left = plot_right;
                let axis_panel_width = (viewport_width - axis_panel_left).max(0.0);
                let default_text_anchor_x = last_price_label_anchor_x;
                let mut label_text_anchor_x = default_text_anchor_x;
                if style.show_last_price_label_box {
                    let estimated_text_width =
                        estimate_label_text_width_px(&text, style.last_price_label_font_size_px);
                    // Keep width selection deterministic and backend-independent so snapshots
                    // remain stable across null/cairo renderers and CI environments.
                    let requested_box_width = match style.last_price_label_box_width_mode {
                        LastPriceLabelBoxWidthMode::FullAxis => axis_panel_width,
                        LastPriceLabelBoxWidthMode::FitText => (estimated_text_width
                            + 2.0 * style.last_price_label_box_padding_x_px)
                            .max(style.last_price_label_box_min_width_px),
                    };
                    let box_width = requested_box_width.clamp(0.0, axis_panel_width);
                    let box_left = (viewport_width - box_width).max(axis_panel_left);
                    let box_top = (text_y - style.last_price_label_box_padding_y_px)
                        .clamp(0.0, viewport_height);
                    let box_bottom = (text_y
                        + style.last_price_label_font_size_px
                        + style.last_price_label_box_padding_y_px)
                        .clamp(0.0, viewport_height);
                    let box_height = (box_bottom - box_top).max(0.0);
                    label_text_anchor_x = (viewport_width
                        - style.last_price_label_box_padding_x_px)
                        .clamp(box_left, viewport_width);
                    if box_width > 0.0 && box_height > 0.0 {
                        let mut rect = RectPrimitive::new(
                            box_left,
                            box_top,
                            box_width,
                            box_height,
                            box_fill_color,
                        );
                        if style.last_price_label_box_border_width_px > 0.0 {
                            rect = rect.with_border(
                                style.last_price_label_box_border_width_px,
                                style.last_price_label_box_border_color,
                            );
                        }
                        if style.last_price_label_box_corner_radius_px > 0.0 {
                            let max_corner_radius = (box_width.min(box_height)) * 0.5;
                            let clamped_corner_radius = style
                                .last_price_label_box_corner_radius_px
                                .min(max_corner_radius);
                            rect = rect.with_corner_radius(clamped_corner_radius);
                        }
                        frame = frame.with_rect(rect);
                    }
                }
                frame = frame.with_text(TextPrimitive::new(
                    text,
                    if style.show_last_price_label_box {
                        label_text_anchor_x
                    } else {
                        default_text_anchor_x
                    },
                    text_y,
                    style.last_price_label_font_size_px,
                    label_text_color,
                    TextHAlign::Right,
                ));
            }
        }

        let crosshair = self.interaction.crosshair();
        if crosshair.visible {
            let crosshair_x = crosshair
                .snapped_x
                .unwrap_or(crosshair.x)
                .clamp(0.0, plot_right);
            let crosshair_y = crosshair
                .snapped_y
                .unwrap_or(crosshair.y)
                .clamp(0.0, plot_bottom);
            let mut time_box_rect: Option<RectPrimitive> = None;
            let mut time_box_text: Option<TextPrimitive> = None;
            let mut price_box_rect: Option<RectPrimitive> = None;
            let mut price_box_text: Option<TextPrimitive> = None;
            if style.show_crosshair_lines && style.show_crosshair_vertical_line {
                let vertical_line_color = style
                    .crosshair_vertical_line_color
                    .unwrap_or(style.crosshair_line_color);
                let vertical_line_width = style
                    .crosshair_vertical_line_width
                    .unwrap_or(style.crosshair_line_width);
                frame = frame.with_line(
                    LinePrimitive::new(
                        crosshair_x,
                        0.0,
                        crosshair_x,
                        plot_bottom,
                        vertical_line_width,
                        vertical_line_color,
                    )
                    .with_stroke_style(
                        style
                            .crosshair_vertical_line_style
                            .unwrap_or(style.crosshair_line_style),
                    ),
                );
            }
            if style.show_crosshair_lines && style.show_crosshair_horizontal_line {
                let horizontal_line_color = style
                    .crosshair_horizontal_line_color
                    .unwrap_or(style.crosshair_line_color);
                let horizontal_line_width = style
                    .crosshair_horizontal_line_width
                    .unwrap_or(style.crosshair_line_width);
                frame = frame.with_line(
                    LinePrimitive::new(
                        0.0,
                        crosshair_y,
                        plot_right,
                        crosshair_y,
                        horizontal_line_width,
                        horizontal_line_color,
                    )
                    .with_stroke_style(
                        style
                            .crosshair_horizontal_line_style
                            .unwrap_or(style.crosshair_line_style),
                    ),
                );
            }
            if style.show_crosshair_time_label {
                let time_box_fill_color = style
                    .crosshair_time_label_box_color
                    .unwrap_or(style.crosshair_label_box_color);
                let crosshair_time = crosshair
                    .snapped_time
                    .unwrap_or(self.time_scale.pixel_to_time(crosshair_x, self.viewport)?);
                let time_label_padding_x = style
                    .crosshair_time_label_padding_x_px
                    .clamp(0.0, plot_right * 0.5);
                let crosshair_time_label_x = crosshair_x.clamp(
                    time_label_padding_x,
                    (plot_right - time_label_padding_x).max(time_label_padding_x),
                );
                let time_stabilization_step =
                    if style.crosshair_time_label_box_stabilization_step_px > 0.0 {
                        style.crosshair_time_label_box_stabilization_step_px
                    } else {
                        style.crosshair_label_box_stabilization_step_px
                    };
                let crosshair_time_label_x =
                    stabilize_position(crosshair_time_label_x, time_stabilization_step).clamp(
                        time_label_padding_x,
                        (plot_right - time_label_padding_x).max(time_label_padding_x),
                    );
                let mut time_text_x = crosshair_time_label_x;
                let mut time_text_h_align = TextHAlign::Center;
                let text = if let Some(formatter) = &self.crosshair_time_label_formatter {
                    formatter(crosshair_time)
                } else {
                    self.format_time_axis_label(crosshair_time, visible_span_abs)
                };
                let time_label_anchor_y = (plot_bottom + style.crosshair_time_label_offset_y_px)
                    .min((viewport_height - style.crosshair_time_label_font_size_px).max(0.0));
                let mut time_label_y = time_label_anchor_y;
                let time_label_text_color = if style.show_crosshair_time_label_box {
                    self.resolve_crosshair_label_box_text_color(
                        style.crosshair_time_label_color,
                        time_box_fill_color,
                        style.crosshair_time_label_box_text_color,
                        style.crosshair_time_label_box_auto_text_contrast,
                    )
                } else {
                    style.crosshair_time_label_color
                };
                if style.show_crosshair_time_label_box {
                    time_text_h_align = style
                        .crosshair_time_label_box_text_h_align
                        .or(style.crosshair_label_box_text_h_align)
                        .unwrap_or(TextHAlign::Center);
                    let estimated_text_width = estimate_label_text_width_px(
                        &text,
                        style.crosshair_time_label_font_size_px,
                    );
                    let time_box_width_mode = style
                        .crosshair_time_label_box_width_mode
                        .unwrap_or(style.crosshair_label_box_width_mode);
                    let time_box_min_width = if style.crosshair_time_label_box_min_width_px > 0.0 {
                        style.crosshair_time_label_box_min_width_px
                    } else {
                        style.crosshair_label_box_min_width_px
                    };
                    let time_box_vertical_anchor = style
                        .crosshair_time_label_box_vertical_anchor
                        .unwrap_or(style.crosshair_label_box_vertical_anchor);
                    let time_box_overflow_policy = style
                        .crosshair_time_label_box_overflow_policy
                        .or(style.crosshair_label_box_overflow_policy)
                        .unwrap_or(CrosshairLabelBoxOverflowPolicy::ClipToAxis);
                    let time_box_clip_margin =
                        if style.crosshair_time_label_box_clip_margin_px > 0.0 {
                            style.crosshair_time_label_box_clip_margin_px
                        } else {
                            style.crosshair_label_box_clip_margin_px
                        };
                    let time_clip_min_x = if time_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        time_box_clip_margin.min(plot_right * 0.5)
                    } else {
                        0.0
                    };
                    let time_clip_max_x = if time_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        (plot_right - time_box_clip_margin).max(time_clip_min_x)
                    } else {
                        plot_right
                    };
                    let time_clip_min_y = if time_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        let axis_height = (viewport_height - plot_bottom).max(0.0);
                        plot_bottom + time_box_clip_margin.min(axis_height * 0.5)
                    } else {
                        plot_bottom
                    };
                    let time_clip_max_y = if time_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        (viewport_height - time_box_clip_margin).max(time_clip_min_y)
                    } else {
                        viewport_height
                    };
                    let requested_box_width = match time_box_width_mode {
                        CrosshairLabelBoxWidthMode::FullAxis => plot_right,
                        CrosshairLabelBoxWidthMode::FitText => {
                            estimated_text_width + 2.0 * style.crosshair_time_label_box_padding_x_px
                        }
                    };
                    let time_max_box_width = (time_clip_max_x - time_clip_min_x).max(0.0);
                    let box_width = requested_box_width
                        .max(time_box_min_width)
                        .clamp(0.0, time_max_box_width);
                    let time_box_horizontal_anchor = style
                        .crosshair_time_label_box_horizontal_anchor
                        .or(style.crosshair_label_box_horizontal_anchor)
                        .unwrap_or(CrosshairLabelBoxHorizontalAnchor::Center);
                    let max_left = (time_clip_max_x - box_width).max(time_clip_min_x);
                    let requested_left = match time_box_horizontal_anchor {
                        CrosshairLabelBoxHorizontalAnchor::Left => crosshair_time_label_x,
                        CrosshairLabelBoxHorizontalAnchor::Center => {
                            crosshair_time_label_x - box_width * 0.5
                        }
                        CrosshairLabelBoxHorizontalAnchor::Right => {
                            crosshair_time_label_x - box_width
                        }
                    };
                    let box_left = if time_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        requested_left.clamp(time_clip_min_x, max_left)
                    } else {
                        requested_left
                    };
                    let (resolved_time_label_y, box_top, box_bottom) =
                        resolve_crosshair_box_vertical_layout(
                            time_label_anchor_y,
                            style.crosshair_time_label_font_size_px,
                            style.crosshair_time_label_box_padding_y_px,
                            time_clip_min_y,
                            time_clip_max_y,
                            time_box_vertical_anchor,
                            time_box_overflow_policy == CrosshairLabelBoxOverflowPolicy::ClipToAxis,
                        );
                    time_label_y = resolved_time_label_y;
                    let box_height = (box_bottom - box_top).max(0.0);
                    if box_width > 0.0 && box_height > 0.0 {
                        time_text_x = match time_text_h_align {
                            TextHAlign::Left => (box_left
                                + style.crosshair_time_label_box_padding_x_px)
                                .clamp(box_left, box_left + box_width),
                            TextHAlign::Center => box_left + box_width * 0.5,
                            TextHAlign::Right => (box_left + box_width
                                - style.crosshair_time_label_box_padding_x_px)
                                .clamp(box_left, box_left + box_width),
                        };
                        let mut rect = RectPrimitive::new(
                            box_left,
                            box_top,
                            box_width,
                            box_height,
                            time_box_fill_color,
                        );
                        let time_border_width =
                            if style.crosshair_time_label_box_border_width_px > 0.0 {
                                style.crosshair_time_label_box_border_width_px
                            } else {
                                style.crosshair_label_box_border_width_px
                            };
                        let time_border_color =
                            if style.crosshair_time_label_box_border_width_px > 0.0 {
                                style.crosshair_time_label_box_border_color
                            } else {
                                style.crosshair_label_box_border_color
                            };
                        if style.show_crosshair_time_label_box_border && time_border_width > 0.0 {
                            rect = rect.with_border(time_border_width, time_border_color);
                        }
                        let time_corner_radius =
                            if style.crosshair_time_label_box_corner_radius_px > 0.0 {
                                style.crosshair_time_label_box_corner_radius_px
                            } else {
                                style.crosshair_label_box_corner_radius_px
                            };
                        if time_corner_radius > 0.0 {
                            let max_corner_radius = (box_width.min(box_height)) * 0.5;
                            let clamped_corner_radius = time_corner_radius.min(max_corner_radius);
                            rect = rect.with_corner_radius(clamped_corner_radius);
                        }
                        time_box_rect = Some(rect);
                    }
                }
                time_box_text = Some(TextPrimitive::new(
                    text,
                    time_text_x,
                    time_label_y,
                    style.crosshair_time_label_font_size_px,
                    time_label_text_color,
                    time_text_h_align,
                ));
            }
            if style.show_crosshair_price_label {
                let price_box_fill_color = style
                    .crosshair_price_label_box_color
                    .unwrap_or(style.crosshair_label_box_color);
                let crosshair_price = crosshair.snapped_price.unwrap_or(
                    self.price_scale
                        .pixel_to_price(crosshair_y, self.viewport)?,
                );
                let display_price = map_price_to_display_value(
                    crosshair_price,
                    self.price_axis_label_config.display_mode,
                    fallback_display_base_price,
                );
                let text = if let Some(formatter) = &self.crosshair_price_label_formatter {
                    let mut value = formatter(display_price);
                    if !display_suffix.is_empty() {
                        value.push_str(display_suffix);
                    }
                    value
                } else {
                    self.format_price_axis_label(
                        display_price,
                        display_tick_step_abs,
                        display_suffix,
                    )
                };
                let price_label_anchor_y =
                    (crosshair_y - style.crosshair_price_label_offset_y_px).max(0.0);
                let price_stabilization_step =
                    if style.crosshair_price_label_box_stabilization_step_px > 0.0 {
                        style.crosshair_price_label_box_stabilization_step_px
                    } else {
                        style.crosshair_label_box_stabilization_step_px
                    };
                let price_label_anchor_y =
                    stabilize_position(price_label_anchor_y, price_stabilization_step).max(0.0);
                let mut text_y = price_label_anchor_y;
                let price_label_text_color = if style.show_crosshair_price_label_box {
                    self.resolve_crosshair_label_box_text_color(
                        style.crosshair_price_label_color,
                        price_box_fill_color,
                        style.crosshair_price_label_box_text_color,
                        style.crosshair_price_label_box_auto_text_contrast,
                    )
                } else {
                    style.crosshair_price_label_color
                };
                let crosshair_price_label_anchor_x = (viewport_width
                    - style.crosshair_price_label_padding_right_px)
                    .clamp(plot_right, viewport_width);
                let mut text_x = crosshair_price_label_anchor_x;
                let mut price_text_h_align = TextHAlign::Right;
                if style.show_crosshair_price_label_box {
                    price_text_h_align = style
                        .crosshair_price_label_box_text_h_align
                        .or(style.crosshair_label_box_text_h_align)
                        .unwrap_or(TextHAlign::Right);
                    let axis_panel_left = plot_right;
                    let axis_panel_width = (viewport_width - axis_panel_left).max(0.0);
                    let estimated_text_width = estimate_label_text_width_px(
                        &text,
                        style.crosshair_price_label_font_size_px,
                    );
                    let price_box_width_mode = style
                        .crosshair_price_label_box_width_mode
                        .unwrap_or(style.crosshair_label_box_width_mode);
                    let price_box_min_width = if style.crosshair_price_label_box_min_width_px > 0.0
                    {
                        style.crosshair_price_label_box_min_width_px
                    } else {
                        style.crosshair_label_box_min_width_px
                    };
                    let price_box_vertical_anchor = style
                        .crosshair_price_label_box_vertical_anchor
                        .unwrap_or(style.crosshair_label_box_vertical_anchor);
                    let price_box_overflow_policy = style
                        .crosshair_price_label_box_overflow_policy
                        .or(style.crosshair_label_box_overflow_policy)
                        .unwrap_or(CrosshairLabelBoxOverflowPolicy::ClipToAxis);
                    let price_box_clip_margin =
                        if style.crosshair_price_label_box_clip_margin_px > 0.0 {
                            style.crosshair_price_label_box_clip_margin_px
                        } else {
                            style.crosshair_label_box_clip_margin_px
                        };
                    let price_clip_min_x = if price_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        axis_panel_left + price_box_clip_margin.min(axis_panel_width * 0.5)
                    } else {
                        axis_panel_left
                    };
                    let price_clip_max_x = if price_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        (viewport_width - price_box_clip_margin).max(price_clip_min_x)
                    } else {
                        viewport_width
                    };
                    let price_clip_min_y = if price_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        price_box_clip_margin.min(viewport_height * 0.5)
                    } else {
                        0.0
                    };
                    let price_clip_max_y = if price_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        (viewport_height - price_box_clip_margin).max(price_clip_min_y)
                    } else {
                        viewport_height
                    };
                    let requested_box_width = match price_box_width_mode {
                        CrosshairLabelBoxWidthMode::FullAxis => axis_panel_width,
                        CrosshairLabelBoxWidthMode::FitText => {
                            estimated_text_width
                                + 2.0 * style.crosshair_price_label_box_padding_x_px
                        }
                    };
                    let price_max_box_width = (price_clip_max_x - price_clip_min_x).max(0.0);
                    let box_width = requested_box_width
                        .max(price_box_min_width)
                        .clamp(0.0, price_max_box_width);
                    let price_box_horizontal_anchor = style
                        .crosshair_price_label_box_horizontal_anchor
                        .or(style.crosshair_label_box_horizontal_anchor)
                        .unwrap_or(CrosshairLabelBoxHorizontalAnchor::Right);
                    let requested_left = match price_box_horizontal_anchor {
                        CrosshairLabelBoxHorizontalAnchor::Left => axis_panel_left,
                        CrosshairLabelBoxHorizontalAnchor::Center => {
                            axis_panel_left + (axis_panel_width - box_width) * 0.5
                        }
                        CrosshairLabelBoxHorizontalAnchor::Right => viewport_width - box_width,
                    };
                    let box_left = if price_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        requested_left.clamp(
                            price_clip_min_x,
                            (price_clip_max_x - box_width).max(price_clip_min_x),
                        )
                    } else {
                        requested_left
                    };
                    let (resolved_price_label_y, box_top, box_bottom) =
                        resolve_crosshair_box_vertical_layout(
                            price_label_anchor_y,
                            style.crosshair_price_label_font_size_px,
                            style.crosshair_price_label_box_padding_y_px,
                            price_clip_min_y,
                            price_clip_max_y,
                            price_box_vertical_anchor,
                            price_box_overflow_policy
                                == CrosshairLabelBoxOverflowPolicy::ClipToAxis,
                        );
                    text_y = resolved_price_label_y;
                    let box_height = (box_bottom - box_top).max(0.0);
                    text_x = match price_text_h_align {
                        TextHAlign::Left => (box_left
                            + style.crosshair_price_label_box_padding_x_px)
                            .clamp(box_left, box_left + box_width),
                        TextHAlign::Center => box_left + box_width * 0.5,
                        TextHAlign::Right => (box_left + box_width
                            - style.crosshair_price_label_box_padding_x_px)
                            .clamp(box_left, box_left + box_width),
                    };
                    if box_width > 0.0 && box_height > 0.0 {
                        let mut rect = RectPrimitive::new(
                            box_left,
                            box_top,
                            box_width,
                            box_height,
                            price_box_fill_color,
                        );
                        let price_border_width =
                            if style.crosshair_price_label_box_border_width_px > 0.0 {
                                style.crosshair_price_label_box_border_width_px
                            } else {
                                style.crosshair_label_box_border_width_px
                            };
                        let price_border_color =
                            if style.crosshair_price_label_box_border_width_px > 0.0 {
                                style.crosshair_price_label_box_border_color
                            } else {
                                style.crosshair_label_box_border_color
                            };
                        if style.show_crosshair_price_label_box_border && price_border_width > 0.0 {
                            rect = rect.with_border(price_border_width, price_border_color);
                        }
                        let price_corner_radius =
                            if style.crosshair_price_label_box_corner_radius_px > 0.0 {
                                style.crosshair_price_label_box_corner_radius_px
                            } else {
                                style.crosshair_label_box_corner_radius_px
                            };
                        if price_corner_radius > 0.0 {
                            let max_corner_radius = (box_width.min(box_height)) * 0.5;
                            let clamped_corner_radius = price_corner_radius.min(max_corner_radius);
                            rect = rect.with_corner_radius(clamped_corner_radius);
                        }
                        price_box_rect = Some(rect);
                    }
                }
                price_box_text = Some(TextPrimitive::new(
                    text,
                    text_x,
                    text_y,
                    style.crosshair_price_label_font_size_px,
                    price_label_text_color,
                    price_text_h_align,
                ));
            }

            if let (Some(time_rect), Some(price_rect)) = (time_box_rect, price_box_rect) {
                if rects_overlap(time_rect, price_rect) {
                    let time_priority = style
                        .crosshair_time_label_box_visibility_priority
                        .unwrap_or(style.crosshair_label_box_visibility_priority);
                    let price_priority = style
                        .crosshair_price_label_box_visibility_priority
                        .unwrap_or(style.crosshair_label_box_visibility_priority);
                    match (time_priority, price_priority) {
                        (
                            CrosshairLabelBoxVisibilityPriority::PreferTime,
                            CrosshairLabelBoxVisibilityPriority::PreferPrice,
                        ) => {}
                        (CrosshairLabelBoxVisibilityPriority::PreferTime, _) => {
                            price_box_rect = None;
                            price_box_text = None;
                        }
                        (_, CrosshairLabelBoxVisibilityPriority::PreferPrice) => {
                            time_box_rect = None;
                            time_box_text = None;
                        }
                        _ => {}
                    }
                }
            }
            let mut z_order_policy = style.crosshair_label_box_z_order_policy;
            if let Some(time_policy) = style.crosshair_time_label_box_z_order_policy {
                z_order_policy = time_policy;
            }
            if let Some(price_policy) = style.crosshair_price_label_box_z_order_policy {
                z_order_policy = price_policy;
            }
            match z_order_policy {
                CrosshairLabelBoxZOrderPolicy::PriceAboveTime => {
                    if let Some(rect) = time_box_rect {
                        frame = frame.with_rect(rect);
                    }
                    if let Some(rect) = price_box_rect {
                        frame = frame.with_rect(rect);
                    }
                    if let Some(text) = time_box_text {
                        frame = frame.with_text(text);
                    }
                    if let Some(text) = price_box_text {
                        frame = frame.with_text(text);
                    }
                }
                CrosshairLabelBoxZOrderPolicy::TimeAbovePrice => {
                    if let Some(rect) = price_box_rect {
                        frame = frame.with_rect(rect);
                    }
                    if let Some(rect) = time_box_rect {
                        frame = frame.with_rect(rect);
                    }
                    if let Some(text) = price_box_text {
                        frame = frame.with_text(text);
                    }
                    if let Some(text) = time_box_text {
                        frame = frame.with_text(text);
                    }
                }
            }
        }

        frame.validate()?;
        Ok(frame)
    }
}
