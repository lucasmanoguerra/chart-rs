use super::RenderStyle;

pub(super) fn estimate_required_time_axis_height(style: RenderStyle) -> f64 {
    let mut required: f64 = 0.0;
    if style.show_time_axis_tick_marks {
        required = required.max(style.time_axis_tick_mark_length_px);
    }
    if style.show_major_time_tick_marks {
        required = required.max(style.major_time_tick_mark_length_px);
    }
    if style.show_time_axis_labels {
        required =
            required.max(style.time_axis_label_offset_y_px + style.time_axis_label_font_size_px);
    }
    if style.show_major_time_labels {
        required =
            required.max(style.major_time_label_offset_y_px + style.major_time_label_font_size_px);
    }

    // Keep a small deterministic buffer so labels are not glued to panel edges.
    (required + 2.0).max(1.0)
}
