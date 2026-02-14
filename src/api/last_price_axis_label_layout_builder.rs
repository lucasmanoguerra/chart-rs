use crate::render::{Color, RectPrimitive};

use super::layout_helpers::estimate_label_text_width_px;
use super::{LastPriceLabelBoxWidthMode, RenderStyle};

#[derive(Debug, Clone)]
pub(super) struct LastPriceAxisLabelLayout {
    pub text_y: f64,
    pub text_anchor_x: f64,
    pub box_rect: Option<RectPrimitive>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LastPriceAxisLabelLayoutContext<'a> {
    pub marker_py: f64,
    pub text: &'a str,
    pub plot_right: f64,
    pub plot_bottom: f64,
    pub viewport_width: f64,
    pub default_text_anchor_x: f64,
    pub box_fill_color: Color,
    pub style: RenderStyle,
}

pub(super) fn build_last_price_axis_label_layout(
    ctx: LastPriceAxisLabelLayoutContext<'_>,
) -> LastPriceAxisLabelLayout {
    let marker_py = ctx.marker_py;
    let text = ctx.text;
    let plot_right = ctx.plot_right;
    let plot_bottom = ctx.plot_bottom;
    let viewport_width = ctx.viewport_width;
    let default_text_anchor_x = ctx.default_text_anchor_x;
    let box_fill_color = ctx.box_fill_color;
    let style = ctx.style;

    let mut text_y = (marker_py - style.last_price_label_offset_y_px).clamp(
        0.0,
        (plot_bottom - style.last_price_label_font_size_px).max(0.0),
    );
    let axis_panel_left = plot_right;
    let axis_panel_width = (viewport_width - axis_panel_left).max(0.0);
    let mut label_text_anchor_x = default_text_anchor_x;
    let mut box_rect = None;

    if style.show_last_price_label_box {
        let min_text_y = style.last_price_label_box_padding_y_px.max(0.0);
        let max_text_y = (plot_bottom
            - style.last_price_label_font_size_px
            - style.last_price_label_box_padding_y_px.max(0.0))
        .max(min_text_y);
        text_y = text_y.clamp(min_text_y, max_text_y);
        let estimated_text_width =
            estimate_label_text_width_px(text, style.last_price_label_font_size_px);
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
        let box_top = (text_y - style.last_price_label_box_padding_y_px.max(0.0)).max(0.0);
        let box_bottom = (text_y
            + style.last_price_label_font_size_px
            + style.last_price_label_box_padding_y_px.max(0.0))
        .min(plot_bottom);
        let box_height = (box_bottom - box_top).max(0.0);
        label_text_anchor_x = (viewport_width - style.last_price_label_box_padding_x_px)
            .clamp(box_left, viewport_width);
        if box_width > 0.0 && box_height > 0.0 {
            let mut rect =
                RectPrimitive::new(box_left, box_top, box_width, box_height, box_fill_color);
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
            box_rect = Some(rect);
        }
    }

    LastPriceAxisLabelLayout {
        text_y,
        text_anchor_x: if style.show_last_price_label_box {
            label_text_anchor_x
        } else {
            default_text_anchor_x
        },
        box_rect,
    }
}
