use crate::error::{ChartError, ChartResult};

use super::{
    PriceAxisDisplayMode, PriceAxisLabelConfig, PriceAxisLabelPolicy, RenderStyle,
    TimeAxisLabelConfig, TimeAxisLabelPolicy, TimeAxisSessionConfig,
};

pub(super) fn validate_time_axis_label_config(
    config: TimeAxisLabelConfig,
) -> ChartResult<TimeAxisLabelConfig> {
    match config.policy {
        TimeAxisLabelPolicy::LogicalDecimal { precision } => {
            if precision > 12 {
                return Err(ChartError::InvalidData(
                    "time-axis decimal precision must be <= 12".to_owned(),
                ));
            }
        }
        TimeAxisLabelPolicy::UtcDateTime { .. } | TimeAxisLabelPolicy::UtcAdaptive => {}
    }

    let offset_minutes = i32::from(config.timezone.offset_minutes());
    if !(-14 * 60..=14 * 60).contains(&offset_minutes) {
        return Err(ChartError::InvalidData(
            "time-axis timezone offset must be between -840 and 840 minutes".to_owned(),
        ));
    }

    if let Some(session) = config.session {
        validate_time_axis_session_config(session)?;
    }

    Ok(config)
}

pub(super) fn validate_price_axis_label_config(
    config: PriceAxisLabelConfig,
) -> ChartResult<PriceAxisLabelConfig> {
    match config.policy {
        PriceAxisLabelPolicy::FixedDecimals { precision } => {
            if precision > 12 {
                return Err(ChartError::InvalidData(
                    "price-axis decimal precision must be <= 12".to_owned(),
                ));
            }
        }
        PriceAxisLabelPolicy::MinMove { min_move, .. } => {
            if !min_move.is_finite() || min_move <= 0.0 {
                return Err(ChartError::InvalidData(
                    "price-axis min_move must be finite and > 0".to_owned(),
                ));
            }
        }
        PriceAxisLabelPolicy::Adaptive => {}
    }

    match config.display_mode {
        PriceAxisDisplayMode::Normal => {}
        PriceAxisDisplayMode::Percentage { base_price }
        | PriceAxisDisplayMode::IndexedTo100 { base_price } => {
            if let Some(base_price) = base_price {
                if !base_price.is_finite() || base_price == 0.0 {
                    return Err(ChartError::InvalidData(
                        "price-axis display base_price must be finite and != 0".to_owned(),
                    ));
                }
            }
        }
    }

    Ok(config)
}

fn validate_time_axis_session_config(
    session: TimeAxisSessionConfig,
) -> ChartResult<TimeAxisSessionConfig> {
    for (name, value, max_exclusive) in [
        ("start_hour", session.start_hour, 24),
        ("start_minute", session.start_minute, 60),
        ("end_hour", session.end_hour, 24),
        ("end_minute", session.end_minute, 60),
    ] {
        if value >= max_exclusive {
            return Err(ChartError::InvalidData(format!(
                "time-axis session `{name}` must be < {max_exclusive}"
            )));
        }
    }

    if session.start_minute_of_day() == session.end_minute_of_day() {
        return Err(ChartError::InvalidData(
            "time-axis session start/end must not be equal".to_owned(),
        ));
    }

    Ok(session)
}

pub(super) fn validate_render_style(style: RenderStyle) -> ChartResult<RenderStyle> {
    style.series_line_color.validate()?;
    style.grid_line_color.validate()?;
    style.price_axis_grid_line_color.validate()?;
    style.major_grid_line_color.validate()?;
    style.axis_border_color.validate()?;
    style.price_axis_tick_mark_color.validate()?;
    style.time_axis_tick_mark_color.validate()?;
    style.major_time_tick_mark_color.validate()?;
    style.time_axis_label_color.validate()?;
    style.major_time_label_color.validate()?;
    style.axis_label_color.validate()?;
    style.crosshair_line_color.validate()?;
    if let Some(color) = style.crosshair_horizontal_line_color {
        color.validate()?;
    }
    if let Some(color) = style.crosshair_vertical_line_color {
        color.validate()?;
    }
    style.crosshair_time_label_color.validate()?;
    style.crosshair_price_label_color.validate()?;
    style.crosshair_label_box_color.validate()?;
    if let Some(color) = style.crosshair_time_label_box_color {
        color.validate()?;
    }
    if let Some(color) = style.crosshair_price_label_box_color {
        color.validate()?;
    }
    style.crosshair_label_box_text_color.validate()?;
    if let Some(color) = style.crosshair_time_label_box_text_color {
        color.validate()?;
    }
    if let Some(color) = style.crosshair_price_label_box_text_color {
        color.validate()?;
    }
    style.crosshair_label_box_border_color.validate()?;
    style.crosshair_time_label_box_border_color.validate()?;
    style.crosshair_price_label_box_border_color.validate()?;
    style.last_price_line_color.validate()?;
    style.last_price_label_color.validate()?;
    style.last_price_up_color.validate()?;
    style.last_price_down_color.validate()?;
    style.last_price_neutral_color.validate()?;
    style.last_price_label_box_color.validate()?;
    style.last_price_label_box_text_color.validate()?;
    style.last_price_label_box_border_color.validate()?;

    for (name, value) in [
        ("grid_line_width", style.grid_line_width),
        (
            "price_axis_grid_line_width",
            style.price_axis_grid_line_width,
        ),
        ("major_grid_line_width", style.major_grid_line_width),
        ("axis_line_width", style.axis_line_width),
        (
            "price_axis_tick_mark_width",
            style.price_axis_tick_mark_width,
        ),
        ("time_axis_tick_mark_width", style.time_axis_tick_mark_width),
        (
            "major_time_tick_mark_width",
            style.major_time_tick_mark_width,
        ),
        ("crosshair_line_width", style.crosshair_line_width),
        (
            "crosshair_time_label_font_size_px",
            style.crosshair_time_label_font_size_px,
        ),
        (
            "crosshair_price_label_font_size_px",
            style.crosshair_price_label_font_size_px,
        ),
        (
            "crosshair_axis_label_font_size_px",
            style.crosshair_axis_label_font_size_px,
        ),
        ("last_price_line_width", style.last_price_line_width),
        (
            "major_time_label_font_size_px",
            style.major_time_label_font_size_px,
        ),
        (
            "time_axis_label_font_size_px",
            style.time_axis_label_font_size_px,
        ),
        (
            "last_price_label_font_size_px",
            style.last_price_label_font_size_px,
        ),
        (
            "last_price_label_offset_y_px",
            style.last_price_label_offset_y_px,
        ),
        (
            "price_axis_label_font_size_px",
            style.price_axis_label_font_size_px,
        ),
        ("price_axis_width_px", style.price_axis_width_px),
        ("time_axis_height_px", style.time_axis_height_px),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(ChartError::InvalidData(format!(
                "render style `{name}` must be finite and > 0"
            )));
        }
    }
    if let Some(width) = style.crosshair_horizontal_line_width {
        if !width.is_finite() || width <= 0.0 {
            return Err(ChartError::InvalidData(
                "render style `crosshair_horizontal_line_width` must be finite and > 0".to_owned(),
            ));
        }
    }
    if let Some(width) = style.crosshair_vertical_line_width {
        if !width.is_finite() || width <= 0.0 {
            return Err(ChartError::InvalidData(
                "render style `crosshair_vertical_line_width` must be finite and > 0".to_owned(),
            ));
        }
    }
    if !style.price_axis_label_padding_right_px.is_finite()
        || style.price_axis_label_padding_right_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `price_axis_label_padding_right_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.time_axis_label_offset_y_px.is_finite() || style.time_axis_label_offset_y_px < 0.0 {
        return Err(ChartError::InvalidData(
            "render style `time_axis_label_offset_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_time_label_padding_x_px.is_finite()
        || style.crosshair_time_label_padding_x_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_padding_x_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_time_label_offset_y_px.is_finite()
        || style.crosshair_time_label_offset_y_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_offset_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.major_time_label_offset_y_px.is_finite() || style.major_time_label_offset_y_px < 0.0 {
        return Err(ChartError::InvalidData(
            "render style `major_time_label_offset_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.time_axis_tick_mark_length_px.is_finite() || style.time_axis_tick_mark_length_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `time_axis_tick_mark_length_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.major_time_tick_mark_length_px.is_finite()
        || style.major_time_tick_mark_length_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `major_time_tick_mark_length_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.last_price_label_padding_right_px.is_finite()
        || style.last_price_label_padding_right_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_padding_right_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_label_box_padding_x_px.is_finite()
        || style.crosshair_label_box_padding_x_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_label_box_padding_x_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_label_box_padding_y_px.is_finite()
        || style.crosshair_label_box_padding_y_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_label_box_padding_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_label_box_min_width_px.is_finite()
        || style.crosshair_label_box_min_width_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_label_box_min_width_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_time_label_box_min_width_px.is_finite()
        || style.crosshair_time_label_box_min_width_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_box_min_width_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_price_label_box_min_width_px.is_finite()
        || style.crosshair_price_label_box_min_width_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_box_min_width_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_label_box_clip_margin_px.is_finite()
        || style.crosshair_label_box_clip_margin_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_label_box_clip_margin_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_time_label_box_clip_margin_px.is_finite()
        || style.crosshair_time_label_box_clip_margin_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_box_clip_margin_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_price_label_box_clip_margin_px.is_finite()
        || style.crosshair_price_label_box_clip_margin_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_box_clip_margin_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_label_box_stabilization_step_px.is_finite()
        || style.crosshair_label_box_stabilization_step_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_label_box_stabilization_step_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style
        .crosshair_time_label_box_stabilization_step_px
        .is_finite()
        || style.crosshair_time_label_box_stabilization_step_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_box_stabilization_step_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style
        .crosshair_price_label_box_stabilization_step_px
        .is_finite()
        || style.crosshair_price_label_box_stabilization_step_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_box_stabilization_step_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_label_box_border_width_px.is_finite()
        || style.crosshair_label_box_border_width_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_label_box_border_width_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_time_label_box_border_width_px.is_finite()
        || style.crosshair_time_label_box_border_width_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_box_border_width_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_price_label_box_border_width_px.is_finite()
        || style.crosshair_price_label_box_border_width_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_box_border_width_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_label_box_corner_radius_px.is_finite()
        || style.crosshair_label_box_corner_radius_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_label_box_corner_radius_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_time_label_box_corner_radius_px.is_finite()
        || style.crosshair_time_label_box_corner_radius_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_box_corner_radius_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_price_label_box_corner_radius_px.is_finite()
        || style.crosshair_price_label_box_corner_radius_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_box_corner_radius_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_time_label_box_padding_x_px.is_finite()
        || style.crosshair_time_label_box_padding_x_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_box_padding_x_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_time_label_box_padding_y_px.is_finite()
        || style.crosshair_time_label_box_padding_y_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_time_label_box_padding_y_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_price_label_box_padding_x_px.is_finite()
        || style.crosshair_price_label_box_padding_x_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_box_padding_x_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_price_label_box_padding_y_px.is_finite()
        || style.crosshair_price_label_box_padding_y_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_box_padding_y_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.price_axis_tick_mark_length_px.is_finite()
        || style.price_axis_tick_mark_length_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `price_axis_tick_mark_length_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.price_axis_label_offset_y_px.is_finite() || style.price_axis_label_offset_y_px < 0.0 {
        return Err(ChartError::InvalidData(
            "render style `price_axis_label_offset_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.crosshair_price_label_padding_right_px.is_finite()
        || style.crosshair_price_label_padding_right_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_padding_right_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.crosshair_price_label_offset_y_px.is_finite()
        || style.crosshair_price_label_offset_y_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `crosshair_price_label_offset_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.last_price_label_offset_y_px.is_finite() || style.last_price_label_offset_y_px < 0.0 {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_offset_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.last_price_label_exclusion_px.is_finite() || style.last_price_label_exclusion_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_exclusion_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.last_price_label_box_padding_y_px.is_finite()
        || style.last_price_label_box_padding_y_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_box_padding_y_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.last_price_label_box_padding_x_px.is_finite()
        || style.last_price_label_box_padding_x_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_box_padding_x_px` must be finite and >= 0".to_owned(),
        ));
    }
    if !style.last_price_label_box_min_width_px.is_finite()
        || style.last_price_label_box_min_width_px <= 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_box_min_width_px` must be finite and > 0".to_owned(),
        ));
    }
    if !style.last_price_label_box_border_width_px.is_finite()
        || style.last_price_label_box_border_width_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_box_border_width_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    if !style.last_price_label_box_corner_radius_px.is_finite()
        || style.last_price_label_box_corner_radius_px < 0.0
    {
        return Err(ChartError::InvalidData(
            "render style `last_price_label_box_corner_radius_px` must be finite and >= 0"
                .to_owned(),
        ));
    }
    Ok(style)
}
