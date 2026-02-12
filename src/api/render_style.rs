use crate::render::{Color, LineStrokeStyle, TextHAlign};

/// Source policy used for latest-price marker selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LastPriceSourceMode {
    /// Use the newest sample across full series data.
    #[default]
    LatestData,
    /// Use the newest sample that is currently inside visible time range.
    LatestVisible,
}

/// Width policy used for latest-price label box layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LastPriceLabelBoxWidthMode {
    /// Stretch label box to the full axis panel width.
    #[default]
    FullAxis,
    /// Fit label box to text width using configured horizontal padding/min width.
    FitText,
}

/// Width policy used for crosshair axis-label box layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairLabelBoxWidthMode {
    /// Stretch label box to the full axis panel width.
    FullAxis,
    /// Fit label box to text width using configured horizontal padding.
    #[default]
    FitText,
}

/// Vertical anchor used for crosshair axis-label box layout around label Y.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairLabelBoxVerticalAnchor {
    Top,
    #[default]
    Center,
    Bottom,
}

/// Horizontal anchor used for crosshair axis-label box layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairLabelBoxHorizontalAnchor {
    Left,
    #[default]
    Center,
    Right,
}

/// Overflow policy used for crosshair axis-label box layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairLabelBoxOverflowPolicy {
    #[default]
    ClipToAxis,
    AllowOverflow,
}

/// Priority policy used when crosshair time/price label boxes overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairLabelBoxVisibilityPriority {
    #[default]
    KeepBoth,
    PreferTime,
    PreferPrice,
}

/// Z-order policy used when rendering crosshair time/price axis-label boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairLabelBoxZOrderPolicy {
    #[default]
    PriceAboveTime,
    TimeAbovePrice,
}

/// Style contract for the current render frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderStyle {
    pub series_line_color: Color,
    pub grid_line_color: Color,
    pub price_axis_grid_line_color: Color,
    pub major_grid_line_color: Color,
    pub axis_border_color: Color,
    pub price_axis_tick_mark_color: Color,
    pub time_axis_tick_mark_color: Color,
    pub major_time_tick_mark_color: Color,
    pub time_axis_label_color: Color,
    pub major_time_label_color: Color,
    pub axis_label_color: Color,
    pub crosshair_line_color: Color,
    pub crosshair_horizontal_line_color: Option<Color>,
    pub crosshair_vertical_line_color: Option<Color>,
    pub crosshair_time_label_color: Color,
    pub crosshair_price_label_color: Color,
    /// Shared prefix prepended to crosshair axis-label text when per-axis override is absent.
    pub crosshair_label_prefix: &'static str,
    /// Shared suffix appended to crosshair axis-label text when per-axis override is absent.
    pub crosshair_label_suffix: &'static str,
    /// Optional dedicated prefix for crosshair time-axis label text.
    pub crosshair_time_label_prefix: Option<&'static str>,
    /// Optional dedicated suffix for crosshair time-axis label text.
    pub crosshair_time_label_suffix: Option<&'static str>,
    /// Optional dedicated prefix for crosshair price-axis label text.
    pub crosshair_price_label_prefix: Option<&'static str>,
    /// Optional dedicated suffix for crosshair price-axis label text.
    pub crosshair_price_label_suffix: Option<&'static str>,
    /// Shared numeric precision override for crosshair axis labels when per-axis override is absent.
    pub crosshair_label_numeric_precision: Option<u8>,
    /// Optional dedicated numeric precision override for crosshair time-axis labels.
    pub crosshair_time_label_numeric_precision: Option<u8>,
    /// Optional dedicated numeric precision override for crosshair price-axis labels.
    pub crosshair_price_label_numeric_precision: Option<u8>,
    pub crosshair_label_box_color: Color,
    pub crosshair_time_label_box_color: Option<Color>,
    pub crosshair_price_label_box_color: Option<Color>,
    pub crosshair_label_box_text_color: Color,
    pub crosshair_label_box_auto_text_contrast: bool,
    pub crosshair_label_box_text_h_align: Option<TextHAlign>,
    pub crosshair_time_label_box_text_color: Option<Color>,
    pub crosshair_price_label_box_text_color: Option<Color>,
    pub crosshair_time_label_box_auto_text_contrast: Option<bool>,
    pub crosshair_price_label_box_auto_text_contrast: Option<bool>,
    pub crosshair_time_label_box_text_h_align: Option<TextHAlign>,
    pub crosshair_price_label_box_text_h_align: Option<TextHAlign>,
    pub crosshair_label_box_border_color: Color,
    pub crosshair_time_label_box_border_color: Color,
    pub crosshair_price_label_box_border_color: Color,
    pub last_price_line_color: Color,
    pub last_price_label_color: Color,
    /// Applied when trend coloring is enabled and latest sample is above previous.
    pub last_price_up_color: Color,
    /// Applied when trend coloring is enabled and latest sample is below previous.
    pub last_price_down_color: Color,
    /// Applied when trend coloring is enabled and no direction can be inferred.
    pub last_price_neutral_color: Color,
    pub grid_line_width: f64,
    pub price_axis_grid_line_width: f64,
    pub major_grid_line_width: f64,
    pub axis_line_width: f64,
    pub price_axis_tick_mark_width: f64,
    pub time_axis_tick_mark_width: f64,
    pub major_time_tick_mark_width: f64,
    pub crosshair_line_width: f64,
    pub crosshair_horizontal_line_width: Option<f64>,
    pub crosshair_vertical_line_width: Option<f64>,
    pub crosshair_line_style: LineStrokeStyle,
    pub crosshair_horizontal_line_style: Option<LineStrokeStyle>,
    pub crosshair_vertical_line_style: Option<LineStrokeStyle>,
    pub crosshair_time_label_font_size_px: f64,
    pub crosshair_price_label_font_size_px: f64,
    pub crosshair_axis_label_font_size_px: f64,
    pub crosshair_label_box_padding_x_px: f64,
    pub crosshair_label_box_padding_y_px: f64,
    pub crosshair_time_label_box_padding_x_px: f64,
    pub crosshair_time_label_box_padding_y_px: f64,
    pub crosshair_price_label_box_padding_x_px: f64,
    pub crosshair_price_label_box_padding_y_px: f64,
    pub crosshair_label_box_width_mode: CrosshairLabelBoxWidthMode,
    pub crosshair_time_label_box_width_mode: Option<CrosshairLabelBoxWidthMode>,
    pub crosshair_price_label_box_width_mode: Option<CrosshairLabelBoxWidthMode>,
    pub crosshair_label_box_vertical_anchor: CrosshairLabelBoxVerticalAnchor,
    pub crosshair_time_label_box_vertical_anchor: Option<CrosshairLabelBoxVerticalAnchor>,
    pub crosshair_price_label_box_vertical_anchor: Option<CrosshairLabelBoxVerticalAnchor>,
    pub crosshair_label_box_horizontal_anchor: Option<CrosshairLabelBoxHorizontalAnchor>,
    pub crosshair_time_label_box_horizontal_anchor: Option<CrosshairLabelBoxHorizontalAnchor>,
    pub crosshair_price_label_box_horizontal_anchor: Option<CrosshairLabelBoxHorizontalAnchor>,
    pub crosshair_label_box_overflow_policy: Option<CrosshairLabelBoxOverflowPolicy>,
    pub crosshair_time_label_box_overflow_policy: Option<CrosshairLabelBoxOverflowPolicy>,
    pub crosshair_price_label_box_overflow_policy: Option<CrosshairLabelBoxOverflowPolicy>,
    pub crosshair_label_box_clip_margin_px: f64,
    pub crosshair_time_label_box_clip_margin_px: f64,
    pub crosshair_price_label_box_clip_margin_px: f64,
    pub crosshair_label_box_visibility_priority: CrosshairLabelBoxVisibilityPriority,
    pub crosshair_time_label_box_visibility_priority: Option<CrosshairLabelBoxVisibilityPriority>,
    pub crosshair_price_label_box_visibility_priority: Option<CrosshairLabelBoxVisibilityPriority>,
    pub crosshair_label_box_z_order_policy: CrosshairLabelBoxZOrderPolicy,
    pub crosshair_time_label_box_z_order_policy: Option<CrosshairLabelBoxZOrderPolicy>,
    pub crosshair_price_label_box_z_order_policy: Option<CrosshairLabelBoxZOrderPolicy>,
    pub crosshair_label_box_stabilization_step_px: f64,
    pub crosshair_time_label_box_stabilization_step_px: f64,
    pub crosshair_price_label_box_stabilization_step_px: f64,
    pub crosshair_label_box_min_width_px: f64,
    pub crosshair_time_label_box_min_width_px: f64,
    pub crosshair_price_label_box_min_width_px: f64,
    pub crosshair_label_box_border_width_px: f64,
    pub crosshair_time_label_box_border_width_px: f64,
    pub crosshair_price_label_box_border_width_px: f64,
    pub crosshair_label_box_corner_radius_px: f64,
    pub crosshair_time_label_box_corner_radius_px: f64,
    pub crosshair_price_label_box_corner_radius_px: f64,
    pub last_price_line_width: f64,
    pub major_time_label_font_size_px: f64,
    /// Font size used by regular (non-major) time-axis labels.
    pub time_axis_label_font_size_px: f64,
    /// Vertical offset from the plot bottom used by time-axis label anchors.
    pub time_axis_label_offset_y_px: f64,
    /// Vertical offset from the plot bottom used by crosshair time-axis label anchors.
    pub crosshair_time_label_offset_y_px: f64,
    /// Vertical offset from the plot bottom used by major time-axis label anchors.
    pub major_time_label_offset_y_px: f64,
    /// Length of short vertical tick marks extending into the time-axis panel.
    pub time_axis_tick_mark_length_px: f64,
    /// Length of short vertical tick marks for major time-axis ticks.
    pub major_time_tick_mark_length_px: f64,
    pub price_axis_label_font_size_px: f64,
    /// Vertical inset (towards top) applied to price-axis labels from their tick Y position.
    pub price_axis_label_offset_y_px: f64,
    /// Vertical inset (towards top) applied to crosshair price-axis label from crosshair Y.
    pub crosshair_price_label_offset_y_px: f64,
    pub last_price_label_font_size_px: f64,
    /// Vertical inset (towards top) applied to last-price label anchor from marker Y position.
    pub last_price_label_offset_y_px: f64,
    /// Horizontal inset from right edge used by last-price label when box mode is disabled.
    pub last_price_label_padding_right_px: f64,
    pub price_axis_width_px: f64,
    pub time_axis_height_px: f64,
    pub show_price_axis_tick_marks: bool,
    pub show_price_axis_grid_lines: bool,
    pub show_price_axis_labels: bool,
    /// Controls visibility of the right-side price-axis border line.
    pub show_price_axis_border: bool,
    pub show_time_axis_labels: bool,
    /// Controls visibility of the bottom time-axis border line.
    pub show_time_axis_border: bool,
    pub show_major_time_labels: bool,
    pub show_major_time_grid_lines: bool,
    pub show_time_axis_tick_marks: bool,
    /// Controls major time-axis tick-mark visibility independently from regular ticks.
    pub show_major_time_tick_marks: bool,
    /// Controls visibility of the horizontal crosshair guide line.
    pub show_crosshair_horizontal_line: bool,
    /// Controls visibility of the vertical crosshair guide line.
    pub show_crosshair_vertical_line: bool,
    /// Shared visibility gate for crosshair guide lines; per-axis toggles still apply.
    pub show_crosshair_lines: bool,
    /// Controls visibility of the crosshair label projected on the time axis panel.
    pub show_crosshair_time_label: bool,
    /// Controls visibility of the crosshair label projected on the price axis panel.
    pub show_crosshair_price_label: bool,
    /// Controls visibility of the crosshair time-axis label box.
    pub show_crosshair_time_label_box: bool,
    /// Controls visibility of the crosshair price-axis label box.
    pub show_crosshair_price_label_box: bool,
    /// Controls visibility of the border stroke for the crosshair time-axis label box.
    pub show_crosshair_time_label_box_border: bool,
    /// Controls visibility of the border stroke for the crosshair price-axis label box.
    pub show_crosshair_price_label_box_border: bool,
    /// Horizontal inset from left/right plot edges applied to crosshair time-axis label anchor.
    pub crosshair_time_label_padding_x_px: f64,
    /// Horizontal inset from right edge used by crosshair price-axis label when box mode is disabled.
    pub crosshair_price_label_padding_right_px: f64,
    /// Horizontal inset from right edge used by price-axis labels.
    pub price_axis_label_padding_right_px: f64,
    /// Length of short axis tick marks extending into the price-axis panel.
    pub price_axis_tick_mark_length_px: f64,
    pub show_last_price_line: bool,
    pub show_last_price_label: bool,
    /// When enabled, last-price line/label colors are derived from price direction.
    pub last_price_use_trend_color: bool,
    /// Selects whether last-price marker tracks full-series or visible-range latest sample.
    pub last_price_source_mode: LastPriceSourceMode,
    /// Enables a filled price-box background behind last-price axis text.
    pub show_last_price_label_box: bool,
    /// Uses trend/marker color for last-price label box background when enabled.
    pub last_price_label_box_use_marker_color: bool,
    /// Fallback label-box fill color when marker-color mode is disabled.
    pub last_price_label_box_color: Color,
    /// Text color used inside the last-price label box.
    pub last_price_label_box_text_color: Color,
    /// When enabled, text color is derived from label-box fill luminance.
    pub last_price_label_box_auto_text_contrast: bool,
    /// Width mode for latest-price label box layout.
    pub last_price_label_box_width_mode: LastPriceLabelBoxWidthMode,
    /// Horizontal text padding for latest-price label box.
    pub last_price_label_box_padding_x_px: f64,
    /// Vertical padding around last-price text when drawing label box.
    pub last_price_label_box_padding_y_px: f64,
    /// Minimum width for latest-price label box.
    pub last_price_label_box_min_width_px: f64,
    /// Border width for last-price label box.
    pub last_price_label_box_border_width_px: f64,
    /// Border color for last-price label box.
    pub last_price_label_box_border_color: Color,
    /// Corner radius for last-price label box.
    pub last_price_label_box_corner_radius_px: f64,
    pub last_price_label_exclusion_px: f64,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            series_line_color: Color::rgb(0.16, 0.38, 1.0),
            grid_line_color: Color::rgb(0.89, 0.92, 0.95),
            price_axis_grid_line_color: Color::rgb(0.89, 0.92, 0.95),
            major_grid_line_color: Color::rgb(0.78, 0.83, 0.90),
            axis_border_color: Color::rgb(0.82, 0.84, 0.88),
            price_axis_tick_mark_color: Color::rgb(0.82, 0.84, 0.88),
            time_axis_tick_mark_color: Color::rgb(0.82, 0.84, 0.88),
            major_time_tick_mark_color: Color::rgb(0.82, 0.84, 0.88),
            time_axis_label_color: Color::rgb(0.10, 0.12, 0.16),
            major_time_label_color: Color::rgb(0.10, 0.12, 0.16),
            axis_label_color: Color::rgb(0.10, 0.12, 0.16),
            crosshair_line_color: Color::rgb(0.30, 0.35, 0.44),
            crosshair_horizontal_line_color: None,
            crosshair_vertical_line_color: None,
            crosshair_time_label_color: Color::rgb(0.10, 0.12, 0.16),
            crosshair_price_label_color: Color::rgb(0.10, 0.12, 0.16),
            crosshair_label_prefix: "",
            crosshair_label_suffix: "",
            crosshair_time_label_prefix: None,
            crosshair_time_label_suffix: None,
            crosshair_price_label_prefix: None,
            crosshair_price_label_suffix: None,
            crosshair_label_numeric_precision: None,
            crosshair_time_label_numeric_precision: None,
            crosshair_price_label_numeric_precision: None,
            crosshair_label_box_color: Color::rgb(0.94, 0.96, 0.99),
            crosshair_time_label_box_color: None,
            crosshair_price_label_box_color: None,
            crosshair_label_box_text_color: Color::rgb(0.10, 0.12, 0.16),
            crosshair_label_box_auto_text_contrast: false,
            crosshair_label_box_text_h_align: None,
            crosshair_time_label_box_text_color: None,
            crosshair_price_label_box_text_color: None,
            crosshair_time_label_box_auto_text_contrast: None,
            crosshair_price_label_box_auto_text_contrast: None,
            crosshair_time_label_box_text_h_align: None,
            crosshair_price_label_box_text_h_align: None,
            crosshair_label_box_border_color: Color::rgb(0.82, 0.84, 0.88),
            crosshair_time_label_box_border_color: Color::rgb(0.82, 0.84, 0.88),
            crosshair_price_label_box_border_color: Color::rgb(0.82, 0.84, 0.88),
            last_price_line_color: Color::rgb(0.16, 0.38, 1.0),
            last_price_label_color: Color::rgb(0.16, 0.38, 1.0),
            last_price_up_color: Color::rgb(0.06, 0.62, 0.35),
            last_price_down_color: Color::rgb(0.86, 0.22, 0.19),
            last_price_neutral_color: Color::rgb(0.16, 0.38, 1.0),
            grid_line_width: 1.0,
            price_axis_grid_line_width: 1.0,
            major_grid_line_width: 1.25,
            axis_line_width: 1.0,
            price_axis_tick_mark_width: 1.0,
            time_axis_tick_mark_width: 1.0,
            major_time_tick_mark_width: 1.0,
            crosshair_line_width: 1.0,
            crosshair_horizontal_line_width: None,
            crosshair_vertical_line_width: None,
            crosshair_line_style: LineStrokeStyle::Solid,
            crosshair_horizontal_line_style: None,
            crosshair_vertical_line_style: None,
            crosshair_time_label_font_size_px: 11.0,
            crosshair_price_label_font_size_px: 11.0,
            crosshair_axis_label_font_size_px: 11.0,
            crosshair_label_box_padding_x_px: 5.0,
            crosshair_label_box_padding_y_px: 2.0,
            crosshair_time_label_box_padding_x_px: 5.0,
            crosshair_time_label_box_padding_y_px: 2.0,
            crosshair_price_label_box_padding_x_px: 5.0,
            crosshair_price_label_box_padding_y_px: 2.0,
            crosshair_label_box_width_mode: CrosshairLabelBoxWidthMode::FitText,
            crosshair_time_label_box_width_mode: None,
            crosshair_price_label_box_width_mode: None,
            crosshair_label_box_vertical_anchor: CrosshairLabelBoxVerticalAnchor::Center,
            crosshair_time_label_box_vertical_anchor: None,
            crosshair_price_label_box_vertical_anchor: None,
            crosshair_label_box_horizontal_anchor: None,
            crosshair_time_label_box_horizontal_anchor: None,
            crosshair_price_label_box_horizontal_anchor: None,
            crosshair_label_box_overflow_policy: None,
            crosshair_time_label_box_overflow_policy: None,
            crosshair_price_label_box_overflow_policy: None,
            crosshair_label_box_clip_margin_px: 0.0,
            crosshair_time_label_box_clip_margin_px: 0.0,
            crosshair_price_label_box_clip_margin_px: 0.0,
            crosshair_label_box_visibility_priority: CrosshairLabelBoxVisibilityPriority::KeepBoth,
            crosshair_time_label_box_visibility_priority: None,
            crosshair_price_label_box_visibility_priority: None,
            crosshair_label_box_z_order_policy: CrosshairLabelBoxZOrderPolicy::PriceAboveTime,
            crosshair_time_label_box_z_order_policy: None,
            crosshair_price_label_box_z_order_policy: None,
            crosshair_label_box_stabilization_step_px: 0.0,
            crosshair_time_label_box_stabilization_step_px: 0.0,
            crosshair_price_label_box_stabilization_step_px: 0.0,
            crosshair_label_box_min_width_px: 0.0,
            crosshair_time_label_box_min_width_px: 0.0,
            crosshair_price_label_box_min_width_px: 0.0,
            crosshair_label_box_border_width_px: 0.0,
            crosshair_time_label_box_border_width_px: 0.0,
            crosshair_price_label_box_border_width_px: 0.0,
            crosshair_label_box_corner_radius_px: 0.0,
            crosshair_time_label_box_corner_radius_px: 0.0,
            crosshair_price_label_box_corner_radius_px: 0.0,
            last_price_line_width: 1.25,
            major_time_label_font_size_px: 12.0,
            time_axis_label_font_size_px: 11.0,
            time_axis_label_offset_y_px: 4.0,
            crosshair_time_label_offset_y_px: 4.0,
            major_time_label_offset_y_px: 4.0,
            time_axis_tick_mark_length_px: 6.0,
            major_time_tick_mark_length_px: 6.0,
            price_axis_label_font_size_px: 11.0,
            price_axis_label_offset_y_px: 8.0,
            crosshair_price_label_offset_y_px: 8.0,
            last_price_label_font_size_px: 11.0,
            last_price_label_offset_y_px: 7.92,
            last_price_label_padding_right_px: 6.0,
            price_axis_width_px: 72.0,
            time_axis_height_px: 24.0,
            show_price_axis_tick_marks: true,
            show_price_axis_grid_lines: true,
            show_price_axis_labels: true,
            show_price_axis_border: true,
            show_time_axis_labels: true,
            show_time_axis_border: true,
            show_major_time_labels: true,
            show_major_time_grid_lines: true,
            show_time_axis_tick_marks: true,
            show_major_time_tick_marks: true,
            show_crosshair_horizontal_line: true,
            show_crosshair_vertical_line: true,
            show_crosshair_lines: true,
            show_crosshair_time_label: true,
            show_crosshair_price_label: true,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            show_crosshair_time_label_box_border: true,
            show_crosshair_price_label_box_border: true,
            crosshair_time_label_padding_x_px: 0.0,
            crosshair_price_label_padding_right_px: 6.0,
            price_axis_label_padding_right_px: 6.0,
            price_axis_tick_mark_length_px: 6.0,
            show_last_price_line: true,
            show_last_price_label: true,
            last_price_use_trend_color: false,
            last_price_source_mode: LastPriceSourceMode::LatestData,
            show_last_price_label_box: false,
            last_price_label_box_use_marker_color: true,
            last_price_label_box_color: Color::rgb(0.16, 0.38, 1.0),
            last_price_label_box_text_color: Color::rgb(1.0, 1.0, 1.0),
            last_price_label_box_auto_text_contrast: true,
            last_price_label_box_width_mode: LastPriceLabelBoxWidthMode::FullAxis,
            last_price_label_box_padding_x_px: 6.0,
            last_price_label_box_padding_y_px: 2.5,
            last_price_label_box_min_width_px: 42.0,
            last_price_label_box_border_width_px: 0.0,
            last_price_label_box_border_color: Color::rgb(0.82, 0.84, 0.88),
            last_price_label_box_corner_radius_px: 0.0,
            last_price_label_exclusion_px: 22.0,
        }
    }
}
