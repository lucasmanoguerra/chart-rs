use crate::core::PaneId;
use crate::error::ChartResult;
use crate::render::{
    CanvasLayerKind, LayeredRenderFrame, LinePrimitive, RectPrimitive, RenderFrame, Renderer,
    TextHAlign, TextPrimitive,
};

use super::axis_label_format::map_price_to_display_value;
use super::layout_helpers::{
    estimate_label_text_width_px, rects_overlap, resolve_crosshair_box_vertical_layout,
    stabilize_position,
};
use super::{
    ChartEngine, CrosshairLabelBoxHorizontalAnchor, CrosshairLabelBoxOverflowPolicy,
    CrosshairLabelBoxVisibilityPriority, CrosshairLabelBoxWidthMode, CrosshairLabelBoxZOrderPolicy,
    CrosshairLabelSourceMode, RenderStyle,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct CrosshairRenderContext {
    pub main_pane_id: PaneId,
    pub plot_right: f64,
    pub plot_bottom: f64,
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub visible_span_abs: f64,
    pub fallback_display_base_price: f64,
    pub display_tick_step_abs: f64,
    pub display_suffix: &'static str,
    pub style: RenderStyle,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_crosshair_primitives(
        &self,
        frame: &mut RenderFrame,
        layered: &mut LayeredRenderFrame,
        ctx: CrosshairRenderContext,
    ) -> ChartResult<()> {
        let main_pane_id = ctx.main_pane_id;
        let plot_right = ctx.plot_right;
        let plot_bottom = ctx.plot_bottom;
        let viewport_width = ctx.viewport_width;
        let viewport_height = ctx.viewport_height;
        let visible_span_abs = ctx.visible_span_abs;
        let fallback_display_base_price = ctx.fallback_display_base_price;
        let display_tick_step_abs = ctx.display_tick_step_abs;
        let display_suffix = ctx.display_suffix;
        let style = ctx.style;

        macro_rules! push_line {
            ($layer:expr, $line:expr) => {{
                frame.lines.push($line);
                let idx = frame.lines.len() - 1;
                layered.push_line(main_pane_id, $layer, frame.lines[idx]);
            }};
        }
        macro_rules! push_rect {
            ($layer:expr, $rect:expr) => {{
                frame.rects.push($rect);
                let idx = frame.rects.len() - 1;
                layered.push_rect(main_pane_id, $layer, frame.rects[idx]);
            }};
        }
        macro_rules! push_text {
            ($layer:expr, $text:expr) => {{
                frame.texts.push($text);
                let idx = frame.texts.len() - 1;
                layered.push_text(main_pane_id, $layer, frame.texts[idx].clone());
            }};
        }
        let crosshair = self.core.model.interaction.crosshair();
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
                push_line!(
                    CanvasLayerKind::Crosshair,
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
                    )
                );
            }
            if style.show_crosshair_lines && style.show_crosshair_horizontal_line {
                let horizontal_line_color = style
                    .crosshair_horizontal_line_color
                    .unwrap_or(style.crosshair_line_color);
                let horizontal_line_width = style
                    .crosshair_horizontal_line_width
                    .unwrap_or(style.crosshair_line_width);
                push_line!(
                    CanvasLayerKind::Crosshair,
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
                    )
                );
            }
            if style.show_crosshair_time_label {
                let time_box_fill_color = style
                    .crosshair_time_label_box_color
                    .unwrap_or(style.crosshair_label_box_color);
                let crosshair_time = crosshair.snapped_time.unwrap_or(
                    self.core
                        .model
                        .time_scale
                        .pixel_to_time(crosshair_x, self.core.model.viewport)?,
                );
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
                let time_label_precision = style
                    .crosshair_time_label_numeric_precision
                    .or(style.crosshair_label_numeric_precision);
                let time_source_mode = if crosshair.snapped_time.is_some() {
                    CrosshairLabelSourceMode::SnappedData
                } else {
                    CrosshairLabelSourceMode::PointerProjected
                };
                let text = Self::apply_crosshair_label_text_transform(
                    self.format_crosshair_time_axis_label(
                        crosshair_time,
                        visible_span_abs,
                        time_label_precision,
                        time_source_mode,
                    ),
                    style
                        .crosshair_time_label_prefix
                        .unwrap_or(style.crosshair_label_prefix),
                    style
                        .crosshair_time_label_suffix
                        .unwrap_or(style.crosshair_label_suffix),
                );
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
                            true,
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
                    self.core
                        .model
                        .price_scale
                        .pixel_to_price(crosshair_y, self.core.model.viewport)?,
                );
                let display_price = map_price_to_display_value(
                    crosshair_price,
                    self.core.behavior.price_axis_label_config.display_mode,
                    fallback_display_base_price,
                );
                let price_label_precision = style
                    .crosshair_price_label_numeric_precision
                    .or(style.crosshair_label_numeric_precision);
                let price_source_mode = if crosshair.snapped_price.is_some() {
                    CrosshairLabelSourceMode::SnappedData
                } else {
                    CrosshairLabelSourceMode::PointerProjected
                };
                let text = Self::apply_crosshair_label_text_transform(
                    self.format_crosshair_price_axis_label(
                        display_price,
                        display_tick_step_abs,
                        display_suffix,
                        price_label_precision,
                        visible_span_abs,
                        price_source_mode,
                    ),
                    style
                        .crosshair_price_label_prefix
                        .unwrap_or(style.crosshair_label_prefix),
                    style
                        .crosshair_price_label_suffix
                        .unwrap_or(style.crosshair_label_suffix),
                );
                let price_label_anchor_y = (crosshair_y - style.crosshair_price_label_offset_y_px)
                    .clamp(
                        0.0,
                        (plot_bottom - style.crosshair_price_label_font_size_px).max(0.0),
                    );
                let price_stabilization_step =
                    if style.crosshair_price_label_box_stabilization_step_px > 0.0 {
                        style.crosshair_price_label_box_stabilization_step_px
                    } else {
                        style.crosshair_label_box_stabilization_step_px
                    };
                let price_label_anchor_y =
                    stabilize_position(price_label_anchor_y, price_stabilization_step).clamp(
                        0.0,
                        (plot_bottom - style.crosshair_price_label_font_size_px).max(0.0),
                    );
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
                        price_box_clip_margin.min(plot_bottom * 0.5)
                    } else {
                        0.0
                    };
                    let price_clip_max_y = if price_box_overflow_policy
                        == CrosshairLabelBoxOverflowPolicy::ClipToAxis
                    {
                        (plot_bottom - price_box_clip_margin).max(price_clip_min_y)
                    } else {
                        plot_bottom
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
                            true,
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
                        push_rect!(CanvasLayerKind::Axis, rect);
                    }
                    if let Some(rect) = price_box_rect {
                        push_rect!(CanvasLayerKind::Axis, rect);
                    }
                    if let Some(text) = time_box_text {
                        push_text!(CanvasLayerKind::Axis, text);
                    }
                    if let Some(text) = price_box_text {
                        push_text!(CanvasLayerKind::Axis, text);
                    }
                }
                CrosshairLabelBoxZOrderPolicy::TimeAbovePrice => {
                    if let Some(rect) = price_box_rect {
                        push_rect!(CanvasLayerKind::Axis, rect);
                    }
                    if let Some(rect) = time_box_rect {
                        push_rect!(CanvasLayerKind::Axis, rect);
                    }
                    if let Some(text) = price_box_text {
                        push_text!(CanvasLayerKind::Axis, text);
                    }
                    if let Some(text) = time_box_text {
                        push_text!(CanvasLayerKind::Axis, text);
                    }
                }
            }
        }
        Ok(())
    }
}
