use crate::render::RectPrimitive;

use super::CrosshairLabelBoxVerticalAnchor;

pub(super) fn estimate_label_text_width_px(text: &str, font_size_px: f64) -> f64 {
    // Keep this estimate deterministic and backend-independent.
    let units = text.chars().fold(0.0, |acc, ch| {
        acc + match ch {
            '0'..='9' => 0.62,
            '.' | ',' => 0.34,
            '-' | '+' | '%' => 0.42,
            ' ' => 0.33,
            _ => 0.58,
        }
    });
    (units * font_size_px).max(font_size_px)
}

pub(super) fn stabilize_position(value: f64, step_px: f64) -> f64 {
    if step_px > 0.0 {
        (value / step_px).round() * step_px
    } else {
        value
    }
}

pub(super) fn resolve_crosshair_box_vertical_layout(
    label_anchor_y: f64,
    font_size_px: f64,
    padding_y_px: f64,
    min_y: f64,
    max_y: f64,
    anchor: CrosshairLabelBoxVerticalAnchor,
    clip_to_bounds: bool,
) -> (f64, f64, f64) {
    let box_height = (font_size_px + 2.0 * padding_y_px).max(0.0);
    let available_height = (max_y - min_y).max(0.0);
    let clamped_box_height = if clip_to_bounds {
        box_height.min(available_height)
    } else {
        box_height
    };
    let preferred_top = match anchor {
        CrosshairLabelBoxVerticalAnchor::Top => label_anchor_y,
        CrosshairLabelBoxVerticalAnchor::Center => label_anchor_y - padding_y_px,
        CrosshairLabelBoxVerticalAnchor::Bottom => label_anchor_y - clamped_box_height,
    };
    let top = if clip_to_bounds {
        preferred_top.clamp(min_y, max_y - clamped_box_height)
    } else {
        preferred_top
    };
    let bottom = top + clamped_box_height;
    let text_y = match anchor {
        CrosshairLabelBoxVerticalAnchor::Top => top + padding_y_px,
        CrosshairLabelBoxVerticalAnchor::Center => top + (clamped_box_height - font_size_px) * 0.5,
        CrosshairLabelBoxVerticalAnchor::Bottom => {
            top + clamped_box_height - padding_y_px - font_size_px
        }
    };
    let text_y = if clip_to_bounds {
        text_y.clamp(min_y, (max_y - font_size_px).max(min_y))
    } else {
        text_y
    };
    (text_y, top, bottom)
}

pub(super) fn rects_overlap(a: RectPrimitive, b: RectPrimitive) -> bool {
    let a_right = a.x + a.width;
    let a_bottom = a.y + a.height;
    let b_right = b.x + b.width;
    let b_bottom = b.y + b.height;
    a.x < b_right && b.x < a_right && a.y < b_bottom && b.y < a_bottom
}
