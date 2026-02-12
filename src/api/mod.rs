use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Timelike, Utc};
use indexmap::IndexMap;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tracing::{debug, trace};

use crate::core::{
    AreaGeometry, BarGeometry, BaselineGeometry, CandleGeometry, DataPoint, HistogramBar,
    LineSegment, OhlcBar, PriceScale, PriceScaleMode, PriceScaleTuning, TimeScale, TimeScaleTuning,
    Viewport, candles_in_time_window, points_in_time_window, project_area_geometry, project_bars,
    project_baseline_geometry, project_candles, project_histogram_bars, project_line_segments,
};
use crate::error::{ChartError, ChartResult};
use crate::extensions::{
    ChartPlugin, MarkerPlacementConfig, PlacedMarker, PluginContext, PluginEvent, SeriesMarker,
    place_markers_on_candles,
};
use crate::interaction::{
    CrosshairMode, CrosshairSnap, CrosshairState, InteractionMode, InteractionState,
    KineticPanConfig, KineticPanState,
};
use crate::render::{
    Color, LinePrimitive, RectPrimitive, RenderFrame, Renderer, TextHAlign, TextPrimitive,
};

#[cfg(feature = "cairo-backend")]
use crate::render::CairoContextRenderer;

/// Public engine bootstrap configuration.
///
/// This type is serializable so host applications can persist/load chart setup
/// without inventing their own ad-hoc format.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChartEngineConfig {
    pub viewport: Viewport,
    pub time_start: f64,
    pub time_end: f64,
    pub price_min: f64,
    pub price_max: f64,
}

impl ChartEngineConfig {
    /// Creates a minimal config with default price range.
    #[must_use]
    pub fn new(viewport: Viewport, time_start: f64, time_end: f64) -> Self {
        Self {
            viewport,
            time_start,
            time_end,
            price_min: 0.0,
            price_max: 1.0,
        }
    }

    /// Sets initial price domain.
    #[must_use]
    pub fn with_price_domain(mut self, price_min: f64, price_max: f64) -> Self {
        self.price_min = price_min;
        self.price_max = price_max;
        self
    }

    /// Serializes config to pretty JSON for debug/config files.
    pub fn to_json_pretty(self) -> ChartResult<String> {
        serde_json::to_string_pretty(&self)
            .map_err(|e| ChartError::InvalidData(format!("failed to serialize config: {e}")))
    }

    /// Deserializes config from JSON.
    pub fn from_json_str(input: &str) -> ChartResult<Self> {
        serde_json::from_str(input)
            .map_err(|e| ChartError::InvalidData(format!("failed to parse config: {e}")))
    }
}

/// Locale preset used by axis label formatters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AxisLabelLocale {
    #[default]
    EnUs,
    EsEs,
}

/// Built-in policy used for time-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeAxisLabelPolicy {
    /// Render logical time values as decimals.
    LogicalDecimal { precision: u8 },
    /// Interpret logical values as unix timestamps and format in UTC.
    UtcDateTime { show_seconds: bool },
    /// Select UTC format detail based on current visible span (zoom level).
    UtcAdaptive,
}

impl Default for TimeAxisLabelPolicy {
    fn default() -> Self {
        Self::LogicalDecimal { precision: 2 }
    }
}

/// Timezone alignment used by UTC-based time-axis policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TimeAxisTimeZone {
    #[default]
    Utc,
    FixedOffsetMinutes {
        minutes: i16,
    },
}

impl TimeAxisTimeZone {
    #[must_use]
    fn offset_minutes(self) -> i16 {
        match self {
            Self::Utc => 0,
            Self::FixedOffsetMinutes { minutes } => minutes,
        }
    }

    #[must_use]
    fn fixed_offset(self) -> FixedOffset {
        let seconds = i32::from(self.offset_minutes()) * 60;
        FixedOffset::east_opt(seconds)
            .unwrap_or_else(|| FixedOffset::east_opt(0).expect("zero UTC offset is valid"))
    }
}

/// Optional trading-session envelope used by time-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeAxisSessionConfig {
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
}

impl TimeAxisSessionConfig {
    #[must_use]
    fn start_minute_of_day(self) -> u16 {
        u16::from(self.start_hour) * 60 + u16::from(self.start_minute)
    }

    #[must_use]
    fn end_minute_of_day(self) -> u16 {
        u16::from(self.end_hour) * 60 + u16::from(self.end_minute)
    }

    #[must_use]
    fn contains_local_minute(self, minute_of_day: u16) -> bool {
        let start = self.start_minute_of_day();
        let end = self.end_minute_of_day();
        if start < end {
            minute_of_day >= start && minute_of_day <= end
        } else {
            minute_of_day >= start || minute_of_day <= end
        }
    }

    #[must_use]
    fn is_boundary(self, minute_of_day: u16, second: u32) -> bool {
        if second != 0 {
            return false;
        }
        minute_of_day == self.start_minute_of_day() || minute_of_day == self.end_minute_of_day()
    }
}

/// Runtime formatter configuration for the time axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TimeAxisLabelConfig {
    pub locale: AxisLabelLocale,
    pub policy: TimeAxisLabelPolicy,
    pub timezone: TimeAxisTimeZone,
    pub session: Option<TimeAxisSessionConfig>,
}

/// Built-in policy used for price-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PriceAxisLabelPolicy {
    /// Render price values with a fixed number of decimals.
    FixedDecimals { precision: u8 },
    /// Round prices to a deterministic minimum move before formatting.
    MinMove {
        min_move: f64,
        trim_trailing_zeros: bool,
    },
    /// Select precision from current visible price-step density.
    Adaptive,
}

impl Default for PriceAxisLabelPolicy {
    fn default() -> Self {
        Self::FixedDecimals { precision: 2 }
    }
}

/// Display transform used for price-axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum PriceAxisDisplayMode {
    #[default]
    Normal,
    Percentage {
        base_price: Option<f64>,
    },
    IndexedTo100 {
        base_price: Option<f64>,
    },
}

/// Runtime formatter configuration for the price axis.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PriceAxisLabelConfig {
    pub locale: AxisLabelLocale,
    pub policy: PriceAxisLabelPolicy,
    pub display_mode: PriceAxisDisplayMode,
}

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
    pub crosshair_time_label_color: Color,
    pub crosshair_price_label_color: Color,
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
            crosshair_time_label_color: Color::rgb(0.10, 0.12, 0.16),
            crosshair_price_label_color: Color::rgb(0.10, 0.12, 0.16),
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

pub type TimeLabelFormatterFn = Arc<dyn Fn(f64) -> String + Send + Sync + 'static>;
pub type PriceLabelFormatterFn = Arc<dyn Fn(f64) -> String + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeLabelCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

/// Runtime metrics exposed by the in-engine price-label cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PriceLabelCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TimeLabelPattern {
    Date,
    DateMinute,
    DateSecond,
    TimeMinute,
    TimeSecond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PriceLabelCachePolicy {
    FixedDecimals {
        precision: u8,
    },
    MinMove {
        min_move_nanos: i64,
        trim_trailing_zeros: bool,
    },
    Adaptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TimeLabelCacheProfile {
    LogicalDecimal {
        precision: u8,
        locale: AxisLabelLocale,
    },
    Utc {
        locale: AxisLabelLocale,
        pattern: TimeLabelPattern,
        timezone: TimeAxisTimeZone,
        session: Option<TimeAxisSessionConfig>,
    },
    Custom {
        formatter_generation: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PriceLabelCacheProfile {
    BuiltIn {
        locale: AxisLabelLocale,
        policy: PriceLabelCachePolicy,
    },
    Custom {
        formatter_generation: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TimeLabelCacheKey {
    profile: TimeLabelCacheProfile,
    logical_time_millis: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PriceLabelCacheKey {
    profile: PriceLabelCacheProfile,
    display_price_nanos: i64,
    tick_step_nanos: i64,
    has_percent_suffix: bool,
}

#[derive(Debug, Default)]
struct TimeLabelCache {
    entries: HashMap<TimeLabelCacheKey, String>,
    hits: u64,
    misses: u64,
}

#[derive(Debug, Default)]
struct PriceLabelCache {
    entries: HashMap<PriceLabelCacheKey, String>,
    hits: u64,
    misses: u64,
}

impl TimeLabelCache {
    const MAX_ENTRIES: usize = 8192;

    fn get(&mut self, key: TimeLabelCacheKey) -> Option<String> {
        let value = self.entries.get(&key).cloned();
        if value.is_some() {
            self.hits = self.hits.saturating_add(1);
        }
        value
    }

    fn insert(&mut self, key: TimeLabelCacheKey, value: String) {
        self.misses = self.misses.saturating_add(1);
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.clear();
        }
        self.entries.insert(key, value);
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn stats(&self) -> TimeLabelCacheStats {
        TimeLabelCacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.entries.len(),
        }
    }
}

impl PriceLabelCache {
    const MAX_ENTRIES: usize = 8192;

    fn get(&mut self, key: PriceLabelCacheKey) -> Option<String> {
        let value = self.entries.get(&key).cloned();
        if value.is_some() {
            self.hits = self.hits.saturating_add(1);
        }
        value
    }

    fn insert(&mut self, key: PriceLabelCacheKey, value: String) {
        self.misses = self.misses.saturating_add(1);
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.clear();
        }
        self.entries.insert(key, value);
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn stats(&self) -> PriceLabelCacheStats {
        PriceLabelCacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.entries.len(),
        }
    }
}

/// Serializable deterministic state snapshot used by regression tests and
/// debugging tooling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub viewport: Viewport,
    pub time_full_range: (f64, f64),
    pub time_visible_range: (f64, f64),
    pub price_domain: (f64, f64),
    pub crosshair: CrosshairState,
    pub points: Vec<DataPoint>,
    pub candle_geometry: Vec<CandleGeometry>,
    pub series_metadata: IndexMap<String, String>,
}

/// Main orchestration facade consumed by host applications.
///
/// `ChartEngine` coordinates time/price scales, interaction state,
/// data/candle collections, and renderer calls.
pub struct ChartEngine<R: Renderer> {
    renderer: R,
    viewport: Viewport,
    time_scale: TimeScale,
    price_scale: PriceScale,
    price_scale_mode: PriceScaleMode,
    interaction: InteractionState,
    points: Vec<DataPoint>,
    candles: Vec<OhlcBar>,
    series_metadata: IndexMap<String, String>,
    plugins: Vec<Box<dyn ChartPlugin>>,
    time_axis_label_config: TimeAxisLabelConfig,
    price_axis_label_config: PriceAxisLabelConfig,
    time_label_formatter: Option<TimeLabelFormatterFn>,
    price_label_formatter: Option<PriceLabelFormatterFn>,
    time_label_formatter_generation: u64,
    price_label_formatter_generation: u64,
    time_label_cache: RefCell<TimeLabelCache>,
    price_label_cache: RefCell<PriceLabelCache>,
    render_style: RenderStyle,
}

impl<R: Renderer> ChartEngine<R> {
    /// Creates a fully initialized engine with explicit domains.
    pub fn new(renderer: R, config: ChartEngineConfig) -> ChartResult<Self> {
        if !config.viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: config.viewport.width,
                height: config.viewport.height,
            });
        }

        let time_scale = TimeScale::new(config.time_start, config.time_end)?;
        let price_scale = PriceScale::new(config.price_min, config.price_max)?;

        Ok(Self {
            renderer,
            viewport: config.viewport,
            time_scale,
            price_scale,
            price_scale_mode: PriceScaleMode::Linear,
            interaction: InteractionState::default(),
            points: Vec::new(),
            candles: Vec::new(),
            series_metadata: IndexMap::new(),
            plugins: Vec::new(),
            time_axis_label_config: TimeAxisLabelConfig::default(),
            price_axis_label_config: PriceAxisLabelConfig::default(),
            time_label_formatter: None,
            price_label_formatter: None,
            time_label_formatter_generation: 0,
            price_label_formatter_generation: 0,
            time_label_cache: RefCell::new(TimeLabelCache::default()),
            price_label_cache: RefCell::new(PriceLabelCache::default()),
            render_style: RenderStyle::default(),
        })
    }

    /// Replaces line/point data series.
    pub fn set_data(&mut self, points: Vec<DataPoint>) {
        debug!(count = points.len(), "set data points");
        self.points = points;
        self.emit_plugin_event(PluginEvent::DataUpdated {
            points_len: self.points.len(),
        });
    }

    /// Appends a single line/point sample.
    pub fn append_point(&mut self, point: DataPoint) {
        self.points.push(point);
        trace!(count = self.points.len(), "append data point");
        self.emit_plugin_event(PluginEvent::DataUpdated {
            points_len: self.points.len(),
        });
    }

    /// Replaces candlestick series.
    pub fn set_candles(&mut self, candles: Vec<OhlcBar>) {
        debug!(count = candles.len(), "set candles");
        self.candles = candles;
        self.emit_plugin_event(PluginEvent::CandlesUpdated {
            candles_len: self.candles.len(),
        });
    }

    /// Appends a single OHLC bar.
    pub fn append_candle(&mut self, candle: OhlcBar) {
        self.candles.push(candle);
        trace!(count = self.candles.len(), "append candle");
        self.emit_plugin_event(PluginEvent::CandlesUpdated {
            candles_len: self.candles.len(),
        });
    }

    /// Sets or updates deterministic series metadata.
    ///
    /// `IndexMap` is used to preserve insertion order for stable snapshots.
    pub fn set_series_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.series_metadata.insert(key.into(), value.into());
    }

    /// Registers a plugin with unique identifier.
    pub fn register_plugin(&mut self, plugin: Box<dyn ChartPlugin>) -> ChartResult<()> {
        let plugin_id = plugin.id().to_owned();
        if plugin_id.is_empty() {
            return Err(ChartError::InvalidData(
                "plugin id must not be empty".to_owned(),
            ));
        }
        if self.plugins.iter().any(|entry| entry.id() == plugin_id) {
            return Err(ChartError::InvalidData(format!(
                "plugin with id `{plugin_id}` is already registered"
            )));
        }
        self.plugins.push(plugin);
        Ok(())
    }

    /// Unregisters a plugin by id. Returns `true` when removed.
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> bool {
        if let Some(position) = self
            .plugins
            .iter()
            .position(|entry| entry.id() == plugin_id)
        {
            self.plugins.remove(position);
            return true;
        }
        false
    }

    #[must_use]
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    #[must_use]
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.iter().any(|plugin| plugin.id() == plugin_id)
    }

    #[must_use]
    pub fn series_metadata(&self) -> &IndexMap<String, String> {
        &self.series_metadata
    }

    #[must_use]
    pub fn points(&self) -> &[DataPoint] {
        &self.points
    }

    #[must_use]
    pub fn candles(&self) -> &[OhlcBar] {
        &self.candles
    }

    #[must_use]
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// Updates viewport dimensions used by scale mapping and render layout.
    pub fn set_viewport(&mut self, viewport: Viewport) -> ChartResult<()> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }
        self.viewport = viewport;
        Ok(())
    }

    #[must_use]
    pub fn time_axis_label_config(&self) -> TimeAxisLabelConfig {
        self.time_axis_label_config
    }

    pub fn set_time_axis_label_config(&mut self, config: TimeAxisLabelConfig) -> ChartResult<()> {
        validate_time_axis_label_config(config)?;
        self.time_axis_label_config = config;
        self.time_label_cache.borrow_mut().clear();
        Ok(())
    }

    #[must_use]
    pub fn price_axis_label_config(&self) -> PriceAxisLabelConfig {
        self.price_axis_label_config
    }

    pub fn set_price_axis_label_config(&mut self, config: PriceAxisLabelConfig) -> ChartResult<()> {
        validate_price_axis_label_config(config)?;
        self.price_axis_label_config = config;
        self.price_label_cache.borrow_mut().clear();
        Ok(())
    }

    pub fn set_time_label_formatter(&mut self, formatter: TimeLabelFormatterFn) {
        self.time_label_formatter = Some(formatter);
        self.time_label_formatter_generation =
            self.time_label_formatter_generation.saturating_add(1);
        self.time_label_cache.borrow_mut().clear();
    }

    pub fn clear_time_label_formatter(&mut self) {
        self.time_label_formatter = None;
        self.time_label_formatter_generation =
            self.time_label_formatter_generation.saturating_add(1);
        self.time_label_cache.borrow_mut().clear();
    }

    pub fn set_price_label_formatter(&mut self, formatter: PriceLabelFormatterFn) {
        self.price_label_formatter = Some(formatter);
        self.price_label_formatter_generation =
            self.price_label_formatter_generation.saturating_add(1);
        self.price_label_cache.borrow_mut().clear();
    }

    pub fn clear_price_label_formatter(&mut self) {
        self.price_label_formatter = None;
        self.price_label_formatter_generation =
            self.price_label_formatter_generation.saturating_add(1);
        self.price_label_cache.borrow_mut().clear();
    }

    #[must_use]
    pub fn time_label_cache_stats(&self) -> TimeLabelCacheStats {
        self.time_label_cache.borrow().stats()
    }

    pub fn clear_time_label_cache(&self) {
        self.time_label_cache.borrow_mut().clear();
    }

    /// Returns hit/miss counters for the price-axis label cache.
    #[must_use]
    pub fn price_label_cache_stats(&self) -> PriceLabelCacheStats {
        self.price_label_cache.borrow().stats()
    }

    /// Clears cached price-axis label strings.
    pub fn clear_price_label_cache(&self) {
        self.price_label_cache.borrow_mut().clear();
    }

    #[must_use]
    pub fn render_style(&self) -> RenderStyle {
        self.render_style
    }

    pub fn set_render_style(&mut self, style: RenderStyle) -> ChartResult<()> {
        validate_render_style(style)?;
        self.render_style = style;
        Ok(())
    }

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

    fn resolve_price_display_base_price(&self) -> f64 {
        let mut candidate: Option<(f64, f64)> = None;

        for point in &self.points {
            if !point.x.is_finite() || !point.y.is_finite() {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time <= point.x => {
                    Some((best_time, best_price))
                }
                _ => Some((point.x, point.y)),
            };
        }

        for candle in &self.candles {
            if !candle.time.is_finite() || !candle.close.is_finite() {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time <= candle.time => {
                    Some((best_time, best_price))
                }
                _ => Some((candle.time, candle.close)),
            };
        }

        if let Some((_, base_price)) = candidate {
            return base_price;
        }

        let domain = self.price_scale.domain();
        if domain.0.is_finite() { domain.0 } else { 1.0 }
    }

    fn resolve_latest_price_sample_with_window(
        &self,
        window: Option<(f64, f64)>,
    ) -> Option<(f64, f64)> {
        let normalized_window = window.map(|(start, end)| {
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        });
        let mut candidate: Option<(f64, f64)> = None;

        for point in &self.points {
            if !point.x.is_finite() || !point.y.is_finite() {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (point.x < window_start || point.x > window_end)
            {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= point.x => {
                    Some((best_time, best_price))
                }
                _ => Some((point.x, point.y)),
            };
        }

        for candle in &self.candles {
            if !candle.time.is_finite() || !candle.close.is_finite() {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (candle.time < window_start || candle.time > window_end)
            {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= candle.time => {
                    Some((best_time, best_price))
                }
                _ => Some((candle.time, candle.close)),
            };
        }

        candidate
    }

    fn resolve_previous_price_before_time_with_window(
        &self,
        latest_time: f64,
        window: Option<(f64, f64)>,
    ) -> Option<f64> {
        let normalized_window = window.map(|(start, end)| {
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        });
        let mut candidate: Option<(f64, f64)> = None;

        for point in &self.points {
            if !point.x.is_finite() || !point.y.is_finite() || point.x >= latest_time {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (point.x < window_start || point.x > window_end)
            {
                continue;
            }
            // Preserve first-seen winner for equal timestamps to keep frame snapshots stable.
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= point.x => {
                    Some((best_time, best_price))
                }
                _ => Some((point.x, point.y)),
            };
        }

        for candle in &self.candles {
            if !candle.time.is_finite() || !candle.close.is_finite() || candle.time >= latest_time {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (candle.time < window_start || candle.time > window_end)
            {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= candle.time => {
                    Some((best_time, best_price))
                }
                _ => Some((candle.time, candle.close)),
            };
        }

        candidate.map(|(_, price)| price)
    }

    fn resolve_latest_and_previous_price_values(
        &self,
        source_mode: LastPriceSourceMode,
        visible_start: f64,
        visible_end: f64,
    ) -> Option<(f64, Option<f64>)> {
        let window = match source_mode {
            LastPriceSourceMode::LatestData => None,
            LastPriceSourceMode::LatestVisible => Some((visible_start, visible_end)),
        };
        let (latest_time, latest_price) = self.resolve_latest_price_sample_with_window(window)?;
        let previous_price =
            self.resolve_previous_price_before_time_with_window(latest_time, window);
        Some((latest_price, previous_price))
    }

    fn resolve_last_price_marker_colors(
        &self,
        latest_price: f64,
        previous_price: Option<f64>,
    ) -> (Color, Color) {
        let style = self.render_style;
        if !style.last_price_use_trend_color {
            return (style.last_price_line_color, style.last_price_label_color);
        }

        let trend_color = match previous_price {
            Some(previous) if latest_price > previous => style.last_price_up_color,
            Some(previous) if latest_price < previous => style.last_price_down_color,
            _ => style.last_price_neutral_color,
        };
        (trend_color, trend_color)
    }

    fn resolve_last_price_label_box_fill_color(&self, marker_label_color: Color) -> Color {
        let style = self.render_style;
        if style.last_price_label_box_use_marker_color {
            marker_label_color
        } else {
            style.last_price_label_box_color
        }
    }

    fn resolve_last_price_label_box_text_color(
        &self,
        box_fill_color: Color,
        marker_label_color: Color,
    ) -> Color {
        let style = self.render_style;
        if !style.show_last_price_label_box {
            return marker_label_color;
        }
        if !style.last_price_label_box_auto_text_contrast {
            return style.last_price_label_box_text_color;
        }

        Self::resolve_auto_contrast_text_color(box_fill_color)
    }

    fn resolve_crosshair_label_box_text_color(
        &self,
        fallback_text_color: Color,
        box_fill_color: Color,
        per_axis_text_color: Option<Color>,
        per_axis_auto_contrast: Option<bool>,
    ) -> Color {
        let style = self.render_style;
        let auto_contrast =
            per_axis_auto_contrast.unwrap_or(style.crosshair_label_box_auto_text_contrast);
        if !auto_contrast {
            return per_axis_text_color.unwrap_or(style.crosshair_label_box_text_color);
        }
        if !style.show_crosshair_time_label_box && !style.show_crosshair_price_label_box {
            return fallback_text_color;
        }

        Self::resolve_auto_contrast_text_color(box_fill_color)
    }

    fn resolve_auto_contrast_text_color(box_fill_color: Color) -> Color {
        // WCAG-inspired luminance gate keeps axis text readable on dynamic marker fills.
        let luminance = 0.2126 * box_fill_color.red
            + 0.7152 * box_fill_color.green
            + 0.0722 * box_fill_color.blue;
        if luminance >= 0.56 {
            Color::rgb(0.06, 0.08, 0.11)
        } else {
            Color::rgb(1.0, 1.0, 1.0)
        }
    }

    fn estimate_label_text_width_px(text: &str, font_size_px: f64) -> f64 {
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

    fn stabilize_position(value: f64, step_px: f64) -> f64 {
        if step_px > 0.0 {
            (value / step_px).round() * step_px
        } else {
            value
        }
    }

    fn resolve_crosshair_box_vertical_layout(
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
            CrosshairLabelBoxVerticalAnchor::Center => {
                top + (clamped_box_height - font_size_px) * 0.5
            }
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

    fn rects_overlap(a: RectPrimitive, b: RectPrimitive) -> bool {
        let a_right = a.x + a.width;
        let a_bottom = a.y + a.height;
        let b_right = b.x + b.width;
        let b_bottom = b.y + b.height;
        a.x < b_right && b.x < a_right && a.y < b_bottom && b.y < a_bottom
    }

    fn resolve_time_label_cache_profile(&self, visible_span_abs: f64) -> TimeLabelCacheProfile {
        if self.time_label_formatter.is_some() {
            return TimeLabelCacheProfile::Custom {
                formatter_generation: self.time_label_formatter_generation,
            };
        }

        match resolve_time_label_pattern(self.time_axis_label_config.policy, visible_span_abs) {
            ResolvedTimeLabelPattern::LogicalDecimal { precision } => {
                TimeLabelCacheProfile::LogicalDecimal {
                    precision,
                    locale: self.time_axis_label_config.locale,
                }
            }
            ResolvedTimeLabelPattern::Utc { pattern } => TimeLabelCacheProfile::Utc {
                locale: self.time_axis_label_config.locale,
                pattern,
                timezone: self.time_axis_label_config.timezone,
                session: self.time_axis_label_config.session,
            },
        }
    }

    fn resolve_price_label_cache_profile(&self) -> PriceLabelCacheProfile {
        if self.price_label_formatter.is_some() {
            return PriceLabelCacheProfile::Custom {
                formatter_generation: self.price_label_formatter_generation,
            };
        }

        PriceLabelCacheProfile::BuiltIn {
            locale: self.price_axis_label_config.locale,
            policy: price_policy_profile(self.price_axis_label_config.policy),
        }
    }

    #[must_use]
    pub fn interaction_mode(&self) -> InteractionMode {
        self.interaction.mode()
    }

    #[must_use]
    pub fn crosshair_mode(&self) -> CrosshairMode {
        self.interaction.crosshair_mode()
    }

    pub fn set_crosshair_mode(&mut self, mode: CrosshairMode) {
        self.interaction.set_crosshair_mode(mode);
    }

    #[must_use]
    pub fn kinetic_pan_config(&self) -> KineticPanConfig {
        self.interaction.kinetic_pan_config()
    }

    pub fn set_kinetic_pan_config(&mut self, config: KineticPanConfig) -> ChartResult<()> {
        validate_kinetic_pan_config(config)?;
        self.interaction.set_kinetic_pan_config(config);
        Ok(())
    }

    #[must_use]
    pub fn kinetic_pan_state(&self) -> KineticPanState {
        self.interaction.kinetic_pan_state()
    }

    /// Starts kinetic pan with signed velocity in time-units per second.
    pub fn start_kinetic_pan(&mut self, velocity_time_per_sec: f64) -> ChartResult<()> {
        if !velocity_time_per_sec.is_finite() {
            return Err(ChartError::InvalidData(
                "kinetic pan velocity must be finite".to_owned(),
            ));
        }
        if velocity_time_per_sec == 0.0 {
            self.stop_kinetic_pan();
            return Ok(());
        }
        self.interaction.start_kinetic_pan(velocity_time_per_sec);
        self.emit_plugin_event(PluginEvent::PanStarted);
        Ok(())
    }

    pub fn stop_kinetic_pan(&mut self) {
        if self.interaction.kinetic_pan_state().active {
            self.interaction.stop_kinetic_pan();
            self.emit_plugin_event(PluginEvent::PanEnded);
        }
    }

    #[must_use]
    pub fn crosshair_state(&self) -> CrosshairState {
        self.interaction.crosshair()
    }

    /// Handles pointer movement and updates crosshair snapping in one step.
    pub fn pointer_move(&mut self, x: f64, y: f64) {
        self.interaction.on_pointer_move(x, y);
        match self.interaction.crosshair_mode() {
            CrosshairMode::Magnet => self.interaction.set_crosshair_snap(self.snap_at_x(x)),
            CrosshairMode::Normal => self.interaction.set_crosshair_snap(None),
        }
        self.emit_plugin_event(PluginEvent::PointerMoved { x, y });
    }

    /// Marks pointer as outside chart bounds.
    pub fn pointer_leave(&mut self) {
        self.interaction.on_pointer_leave();
        self.emit_plugin_event(PluginEvent::PointerLeft);
    }

    pub fn pan_start(&mut self) {
        self.interaction.on_pan_start();
        self.emit_plugin_event(PluginEvent::PanStarted);
    }

    pub fn pan_end(&mut self) {
        self.interaction.on_pan_end();
        self.emit_plugin_event(PluginEvent::PanEnded);
    }

    pub fn map_x_to_pixel(&self, x: f64) -> ChartResult<f64> {
        self.time_scale.time_to_pixel(x, self.viewport)
    }

    pub fn map_pixel_to_x(&self, pixel: f64) -> ChartResult<f64> {
        self.time_scale.pixel_to_time(pixel, self.viewport)
    }

    #[must_use]
    pub fn time_visible_range(&self) -> (f64, f64) {
        self.time_scale.visible_range()
    }

    #[must_use]
    pub fn time_full_range(&self) -> (f64, f64) {
        self.time_scale.full_range()
    }

    /// Returns point samples currently inside the visible time window.
    #[must_use]
    pub fn visible_points(&self) -> Vec<DataPoint> {
        let (start, end) = self.time_scale.visible_range();
        points_in_time_window(&self.points, start, end)
    }

    /// Returns candle samples currently inside the visible time window.
    #[must_use]
    pub fn visible_candles(&self) -> Vec<OhlcBar> {
        let (start, end) = self.time_scale.visible_range();
        candles_in_time_window(&self.candles, start, end)
    }

    /// Returns visible points with symmetric overscan around the visible window.
    pub fn visible_points_with_overscan(&self, ratio: f64) -> ChartResult<Vec<DataPoint>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        Ok(points_in_time_window(&self.points, start, end))
    }

    /// Returns visible candles with symmetric overscan around the visible window.
    pub fn visible_candles_with_overscan(&self, ratio: f64) -> ChartResult<Vec<OhlcBar>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        Ok(candles_in_time_window(&self.candles, start, end))
    }

    /// Overrides visible time range (zoom/pan style behavior).
    pub fn set_time_visible_range(&mut self, start: f64, end: f64) -> ChartResult<()> {
        self.time_scale.set_visible_range(start, end)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Resets visible range to fitted full range.
    pub fn reset_time_visible_range(&mut self) {
        self.time_scale.reset_visible_range_to_full();
        self.emit_visible_range_changed();
    }

    /// Pans visible range by explicit time delta.
    pub fn pan_time_visible_by(&mut self, delta_time: f64) -> ChartResult<()> {
        self.time_scale.pan_visible_by_delta(delta_time)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Pans visible range using pixel drag delta.
    ///
    /// Positive `delta_px` moves the range to earlier times, matching common
    /// drag-to-scroll chart behavior.
    pub fn pan_time_visible_by_pixels(&mut self, delta_px: f64) -> ChartResult<()> {
        if !delta_px.is_finite() {
            return Err(ChartError::InvalidData(
                "pan pixel delta must be finite".to_owned(),
            ));
        }

        let (start, end) = self.time_scale.visible_range();
        let span = end - start;
        let delta_time = -(delta_px / f64::from(self.viewport.width)) * span;
        self.time_scale.pan_visible_by_delta(delta_time)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Applies wheel-driven horizontal pan.
    ///
    /// Conventions:
    /// - one wheel notch is normalized as `120` units
    /// - `wheel_delta_x > 0` pans to later times
    ///
    /// Returns the applied time displacement.
    pub fn wheel_pan_time_visible(
        &mut self,
        wheel_delta_x: f64,
        pan_step_ratio: f64,
    ) -> ChartResult<f64> {
        if !wheel_delta_x.is_finite() {
            return Err(ChartError::InvalidData(
                "wheel pan delta must be finite".to_owned(),
            ));
        }
        if !pan_step_ratio.is_finite() || pan_step_ratio <= 0.0 {
            return Err(ChartError::InvalidData(
                "wheel pan step ratio must be finite and > 0".to_owned(),
            ));
        }
        if wheel_delta_x == 0.0 {
            return Ok(0.0);
        }

        let (start, end) = self.time_scale.visible_range();
        let span = end - start;
        let normalized_steps = wheel_delta_x / 120.0;
        let delta_time = normalized_steps * span * pan_step_ratio;
        self.pan_time_visible_by(delta_time)?;
        Ok(delta_time)
    }

    /// Zooms visible range around a logical time anchor.
    pub fn zoom_time_visible_around_time(
        &mut self,
        factor: f64,
        anchor_time: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        self.time_scale
            .zoom_visible_by_factor(factor, anchor_time, min_span_absolute)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Zooms visible range around a pixel anchor.
    pub fn zoom_time_visible_around_pixel(
        &mut self,
        factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        let anchor_time = self.map_pixel_to_x(anchor_px)?;
        self.time_scale
            .zoom_visible_by_factor(factor, anchor_time, min_span_absolute)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Applies wheel-driven zoom around a pixel anchor.
    ///
    /// Conventions:
    /// - `wheel_delta_y < 0` zooms in
    /// - `wheel_delta_y > 0` zooms out
    /// - one wheel notch is normalized as `120` units
    ///
    /// Returns the effective zoom factor applied to the visible range.
    pub fn wheel_zoom_time_visible(
        &mut self,
        wheel_delta_y: f64,
        anchor_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        if !wheel_delta_y.is_finite() {
            return Err(ChartError::InvalidData(
                "wheel delta must be finite".to_owned(),
            ));
        }
        if !zoom_step_ratio.is_finite() || zoom_step_ratio <= 0.0 {
            return Err(ChartError::InvalidData(
                "wheel zoom step ratio must be finite and > 0".to_owned(),
            ));
        }
        if wheel_delta_y == 0.0 {
            return Ok(1.0);
        }

        let normalized_steps = wheel_delta_y / 120.0;
        let base = 1.0 + zoom_step_ratio;
        let factor = base.powf(-normalized_steps);
        if !factor.is_finite() || factor <= 0.0 {
            return Err(ChartError::InvalidData(
                "computed wheel zoom factor must be finite and > 0".to_owned(),
            ));
        }

        self.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
        Ok(factor)
    }

    /// Advances kinetic pan by a deterministic simulation step.
    ///
    /// Returns `true` when a displacement was applied.
    pub fn step_kinetic_pan(&mut self, delta_seconds: f64) -> ChartResult<bool> {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return Err(ChartError::InvalidData(
                "kinetic pan delta seconds must be finite and > 0".to_owned(),
            ));
        }

        let was_active = self.interaction.kinetic_pan_state().active;
        let Some(displacement) = self.interaction.step_kinetic_pan(delta_seconds) else {
            return Ok(false);
        };

        self.pan_time_visible_by(displacement)?;

        if was_active && !self.interaction.kinetic_pan_state().active {
            self.emit_plugin_event(PluginEvent::PanEnded);
        }
        Ok(true)
    }

    /// Fits time scale against available point/candle data.
    pub fn fit_time_to_data(&mut self, tuning: TimeScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() && self.candles.is_empty() {
            return Ok(());
        }

        self.time_scale
            .fit_to_mixed_data(&self.points, &self.candles, tuning)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Maps a raw price value into pixel Y under the active price scale mode.
    pub fn map_price_to_pixel(&self, price: f64) -> ChartResult<f64> {
        self.price_scale.price_to_pixel(price, self.viewport)
    }

    /// Maps a pixel Y coordinate back into a raw price value.
    pub fn map_pixel_to_price(&self, pixel: f64) -> ChartResult<f64> {
        self.price_scale.pixel_to_price(pixel, self.viewport)
    }

    #[must_use]
    pub fn price_domain(&self) -> (f64, f64) {
        self.price_scale.domain()
    }

    /// Returns the active price scale mapping mode.
    #[must_use]
    pub fn price_scale_mode(&self) -> PriceScaleMode {
        self.price_scale_mode
    }

    /// Switches the price scale mapping mode while preserving the current raw domain.
    ///
    /// When switching to `PriceScaleMode::Log`, the current domain must be strictly positive.
    pub fn set_price_scale_mode(&mut self, mode: PriceScaleMode) -> ChartResult<()> {
        self.price_scale = self.price_scale.with_mode(mode)?;
        self.price_scale_mode = mode;
        Ok(())
    }

    pub fn autoscale_price_from_data(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_data_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from points with explicit tuning.
    pub fn autoscale_price_from_data_tuned(&mut self, tuning: PriceScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() {
            return Ok(());
        }
        self.price_scale =
            PriceScale::from_data_tuned_with_mode(&self.points, tuning, self.price_scale_mode)?;
        Ok(())
    }

    pub fn autoscale_price_from_candles(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_candles_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from candles with explicit tuning.
    pub fn autoscale_price_from_candles_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        if self.candles.is_empty() {
            return Ok(());
        }
        self.price_scale =
            PriceScale::from_ohlc_tuned_with_mode(&self.candles, tuning, self.price_scale_mode)?;
        Ok(())
    }

    pub fn project_candles(&self, body_width_px: f64) -> ChartResult<Vec<CandleGeometry>> {
        project_candles(
            &self.candles,
            self.time_scale,
            self.price_scale,
            self.viewport,
            body_width_px,
        )
    }

    /// Projects only candles inside the active visible time window.
    pub fn project_visible_candles(&self, body_width_px: f64) -> ChartResult<Vec<CandleGeometry>> {
        let (start, end) = self.time_scale.visible_range();
        let visible = candles_in_time_window(&self.candles, start, end);
        project_candles(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            body_width_px,
        )
    }

    /// Projects visible candles with symmetric overscan around the visible range.
    pub fn project_visible_candles_with_overscan(
        &self,
        body_width_px: f64,
        ratio: f64,
    ) -> ChartResult<Vec<CandleGeometry>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        let visible = candles_in_time_window(&self.candles, start, end);
        project_candles(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            body_width_px,
        )
    }

    /// Projects OHLC bars into deterministic bar-series geometry.
    pub fn project_bars(&self, tick_width_px: f64) -> ChartResult<Vec<BarGeometry>> {
        project_bars(
            &self.candles,
            self.time_scale,
            self.price_scale,
            self.viewport,
            tick_width_px,
        )
    }

    /// Projects only bars inside the active visible time window.
    pub fn project_visible_bars(&self, tick_width_px: f64) -> ChartResult<Vec<BarGeometry>> {
        let (start, end) = self.time_scale.visible_range();
        let visible = candles_in_time_window(&self.candles, start, end);
        project_bars(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            tick_width_px,
        )
    }

    /// Projects visible bars with symmetric overscan around the visible range.
    pub fn project_visible_bars_with_overscan(
        &self,
        tick_width_px: f64,
        ratio: f64,
    ) -> ChartResult<Vec<BarGeometry>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        let visible = candles_in_time_window(&self.candles, start, end);
        project_bars(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            tick_width_px,
        )
    }

    /// Projects markers against the full candle set.
    pub fn project_markers_on_candles(
        &self,
        markers: &[SeriesMarker],
        config: MarkerPlacementConfig,
    ) -> ChartResult<Vec<PlacedMarker>> {
        place_markers_on_candles(
            markers,
            &self.candles,
            self.time_scale,
            self.price_scale,
            self.viewport,
            config,
        )
    }

    /// Projects markers against candles in the active visible time window.
    pub fn project_visible_markers_on_candles(
        &self,
        markers: &[SeriesMarker],
        config: MarkerPlacementConfig,
    ) -> ChartResult<Vec<PlacedMarker>> {
        let (start, end) = self.time_scale.visible_range();
        let visible = candles_in_time_window(&self.candles, start, end);
        let visible_markers = markers_in_time_window(markers, start, end);
        place_markers_on_candles(
            &visible_markers,
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            config,
        )
    }

    /// Projects markers against visible candles with symmetric window overscan.
    pub fn project_visible_markers_on_candles_with_overscan(
        &self,
        markers: &[SeriesMarker],
        ratio: f64,
        config: MarkerPlacementConfig,
    ) -> ChartResult<Vec<PlacedMarker>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        let visible = candles_in_time_window(&self.candles, start, end);
        let visible_markers = markers_in_time_window(markers, start, end);
        place_markers_on_candles(
            &visible_markers,
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            config,
        )
    }

    /// Projects line-series points into deterministic segment geometry.
    pub fn project_line_segments(&self) -> ChartResult<Vec<LineSegment>> {
        project_line_segments(
            &self.points,
            self.time_scale,
            self.price_scale,
            self.viewport,
        )
    }

    /// Projects point-series data into deterministic area geometry.
    pub fn project_area_geometry(&self) -> ChartResult<AreaGeometry> {
        project_area_geometry(
            &self.points,
            self.time_scale,
            self.price_scale,
            self.viewport,
        )
    }

    /// Projects only area geometry for points inside the visible time range.
    pub fn project_visible_area_geometry(&self) -> ChartResult<AreaGeometry> {
        let (start, end) = self.time_scale.visible_range();
        let visible = points_in_time_window(&self.points, start, end);
        project_area_geometry(&visible, self.time_scale, self.price_scale, self.viewport)
    }

    /// Projects visible area geometry with symmetric overscan around the window.
    pub fn project_visible_area_geometry_with_overscan(
        &self,
        ratio: f64,
    ) -> ChartResult<AreaGeometry> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        let visible = points_in_time_window(&self.points, start, end);
        project_area_geometry(&visible, self.time_scale, self.price_scale, self.viewport)
    }

    /// Projects point-series data into deterministic baseline geometry.
    pub fn project_baseline_geometry(&self, baseline_price: f64) -> ChartResult<BaselineGeometry> {
        project_baseline_geometry(
            &self.points,
            self.time_scale,
            self.price_scale,
            self.viewport,
            baseline_price,
        )
    }

    /// Projects baseline geometry for points inside the visible time range.
    pub fn project_visible_baseline_geometry(
        &self,
        baseline_price: f64,
    ) -> ChartResult<BaselineGeometry> {
        let (start, end) = self.time_scale.visible_range();
        let visible = points_in_time_window(&self.points, start, end);
        project_baseline_geometry(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            baseline_price,
        )
    }

    /// Projects visible baseline geometry with symmetric window overscan.
    pub fn project_visible_baseline_geometry_with_overscan(
        &self,
        baseline_price: f64,
        ratio: f64,
    ) -> ChartResult<BaselineGeometry> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        let visible = points_in_time_window(&self.points, start, end);
        project_baseline_geometry(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            baseline_price,
        )
    }

    /// Projects point-series data into deterministic histogram bars.
    pub fn project_histogram_bars(
        &self,
        bar_width_px: f64,
        baseline_price: f64,
    ) -> ChartResult<Vec<HistogramBar>> {
        project_histogram_bars(
            &self.points,
            self.time_scale,
            self.price_scale,
            self.viewport,
            bar_width_px,
            baseline_price,
        )
    }

    /// Projects histogram bars for points inside the visible time range.
    pub fn project_visible_histogram_bars(
        &self,
        bar_width_px: f64,
        baseline_price: f64,
    ) -> ChartResult<Vec<HistogramBar>> {
        let (start, end) = self.time_scale.visible_range();
        let visible = points_in_time_window(&self.points, start, end);
        project_histogram_bars(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            bar_width_px,
            baseline_price,
        )
    }

    /// Projects visible histogram bars with symmetric window overscan.
    pub fn project_visible_histogram_bars_with_overscan(
        &self,
        bar_width_px: f64,
        baseline_price: f64,
        ratio: f64,
    ) -> ChartResult<Vec<HistogramBar>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        let visible = points_in_time_window(&self.points, start, end);
        project_histogram_bars(
            &visible,
            self.time_scale,
            self.price_scale,
            self.viewport,
            bar_width_px,
            baseline_price,
        )
    }

    /// Builds a deterministic snapshot useful for regression tests.
    pub fn snapshot(&self, body_width_px: f64) -> ChartResult<EngineSnapshot> {
        Ok(EngineSnapshot {
            viewport: self.viewport,
            time_full_range: self.time_scale.full_range(),
            time_visible_range: self.time_scale.visible_range(),
            price_domain: self.price_scale.domain(),
            crosshair: self.interaction.crosshair(),
            points: self.points.clone(),
            candle_geometry: self.project_candles(body_width_px)?,
            series_metadata: self.series_metadata.clone(),
        })
    }

    /// Serializes snapshot as pretty JSON for fixture-based regression checks.
    pub fn snapshot_json_pretty(&self, body_width_px: f64) -> ChartResult<String> {
        let snapshot = self.snapshot(body_width_px)?;
        serde_json::to_string_pretty(&snapshot)
            .map_err(|e| ChartError::InvalidData(format!("failed to serialize snapshot: {e}")))
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
                    let estimated_text_width = Self::estimate_label_text_width_px(
                        &text,
                        style.last_price_label_font_size_px,
                    );
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
            if style.show_crosshair_vertical_line {
                frame = frame.with_line(LinePrimitive::new(
                    crosshair_x,
                    0.0,
                    crosshair_x,
                    plot_bottom,
                    style.crosshair_line_width,
                    style.crosshair_line_color,
                ));
            }
            if style.show_crosshair_horizontal_line {
                frame = frame.with_line(LinePrimitive::new(
                    0.0,
                    crosshair_y,
                    plot_right,
                    crosshair_y,
                    style.crosshair_line_width,
                    style.crosshair_line_color,
                ));
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
                    Self::stabilize_position(crosshair_time_label_x, time_stabilization_step)
                        .clamp(
                            time_label_padding_x,
                            (plot_right - time_label_padding_x).max(time_label_padding_x),
                        );
                let mut time_text_x = crosshair_time_label_x;
                let mut time_text_h_align = TextHAlign::Center;
                let text = self.format_time_axis_label(crosshair_time, visible_span_abs);
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
                    let estimated_text_width = Self::estimate_label_text_width_px(
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
                        Self::resolve_crosshair_box_vertical_layout(
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
                let text = self.format_price_axis_label(
                    display_price,
                    display_tick_step_abs,
                    display_suffix,
                );
                let price_label_anchor_y =
                    (crosshair_y - style.crosshair_price_label_offset_y_px).max(0.0);
                let price_stabilization_step =
                    if style.crosshair_price_label_box_stabilization_step_px > 0.0 {
                        style.crosshair_price_label_box_stabilization_step_px
                    } else {
                        style.crosshair_label_box_stabilization_step_px
                    };
                let price_label_anchor_y =
                    Self::stabilize_position(price_label_anchor_y, price_stabilization_step)
                        .max(0.0);
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
                    let estimated_text_width = Self::estimate_label_text_width_px(
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
                        Self::resolve_crosshair_box_vertical_layout(
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
                if Self::rects_overlap(time_rect, price_rect) {
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

        frame.validate()?;
        Ok(frame)
    }

    pub fn render(&mut self) -> ChartResult<()> {
        let frame = self.build_render_frame()?;
        self.renderer.render(&frame)?;
        self.emit_plugin_event(PluginEvent::Rendered);
        Ok(())
    }

    /// Renders the frame into an external cairo context.
    ///
    /// This path is used by GTK draw callbacks while keeping the renderer
    /// implementation decoupled from GTK-specific APIs.
    #[cfg(feature = "cairo-backend")]
    pub fn render_on_cairo_context(&mut self, context: &cairo::Context) -> ChartResult<()>
    where
        R: CairoContextRenderer,
    {
        let frame = self.build_render_frame()?;
        self.renderer.render_on_cairo_context(context, &frame)?;
        self.emit_plugin_event(PluginEvent::Rendered);
        Ok(())
    }

    #[must_use]
    pub fn into_renderer(self) -> R {
        self.renderer
    }

    fn plugin_context(&self) -> PluginContext {
        PluginContext {
            viewport: self.viewport,
            time_visible_range: self.time_scale.visible_range(),
            price_domain: self.price_scale.domain(),
            points_len: self.points.len(),
            candles_len: self.candles.len(),
            interaction_mode: self.interaction.mode(),
            crosshair: self.interaction.crosshair(),
        }
    }

    fn emit_plugin_event(&mut self, event: PluginEvent) {
        let context = self.plugin_context();
        for plugin in &mut self.plugins {
            plugin.on_event(event, context);
        }
    }

    fn emit_visible_range_changed(&mut self) {
        let (start, end) = self.time_scale.visible_range();
        self.emit_plugin_event(PluginEvent::VisibleRangeChanged { start, end });
    }

    fn snap_at_x(&self, pointer_x: f64) -> Option<CrosshairSnap> {
        let mut candidates: SmallVec<[(OrderedFloat<f64>, CrosshairSnap); 2]> = SmallVec::new();
        if let Some(snap) = self.nearest_data_snap(pointer_x) {
            candidates.push(snap);
        }
        if let Some(snap) = self.nearest_candle_snap(pointer_x) {
            candidates.push(snap);
        }

        candidates
            .into_iter()
            .min_by_key(|item| item.0)
            .map(|(_, snap)| snap)
    }

    fn nearest_data_snap(&self, pointer_x: f64) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        let mut best: Option<(OrderedFloat<f64>, CrosshairSnap)> = None;
        for point in &self.points {
            let x_px = match self.time_scale.time_to_pixel(point.x, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y_px = match self.price_scale.price_to_pixel(point.y, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dist = OrderedFloat((x_px - pointer_x).abs());
            match best {
                Some((current, _)) if current <= dist => {}
                _ => {
                    best = Some((
                        dist,
                        CrosshairSnap {
                            x: x_px,
                            y: y_px,
                            time: point.x,
                            price: point.y,
                        },
                    ))
                }
            }
        }
        best
    }

    fn nearest_candle_snap(&self, pointer_x: f64) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        let mut best: Option<(OrderedFloat<f64>, CrosshairSnap)> = None;
        for candle in &self.candles {
            let x_px = match self.time_scale.time_to_pixel(candle.time, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y_px = match self.price_scale.price_to_pixel(candle.close, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dist = OrderedFloat((x_px - pointer_x).abs());
            match best {
                Some((current, _)) if current <= dist => {}
                _ => {
                    best = Some((
                        dist,
                        CrosshairSnap {
                            x: x_px,
                            y: y_px,
                            time: candle.time,
                            price: candle.close,
                        },
                    ))
                }
            }
        }
        best
    }
}

fn expand_visible_window(range: (f64, f64), ratio: f64) -> ChartResult<(f64, f64)> {
    if !ratio.is_finite() || ratio < 0.0 {
        return Err(ChartError::InvalidData(
            "overscan ratio must be finite and >= 0".to_owned(),
        ));
    }

    let span = range.1 - range.0;
    let padding = span * ratio;
    Ok((range.0 - padding, range.1 + padding))
}

fn markers_in_time_window(markers: &[SeriesMarker], start: f64, end: f64) -> Vec<SeriesMarker> {
    let (min_t, max_t) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    markers
        .iter()
        .filter(|marker| marker.time >= min_t && marker.time <= max_t)
        .cloned()
        .collect()
}

fn validate_time_axis_label_config(
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

fn validate_price_axis_label_config(
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

fn validate_render_style(style: RenderStyle) -> ChartResult<RenderStyle> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolvedTimeLabelPattern {
    LogicalDecimal { precision: u8 },
    Utc { pattern: TimeLabelPattern },
}

fn resolve_time_label_pattern(
    policy: TimeAxisLabelPolicy,
    visible_span_abs: f64,
) -> ResolvedTimeLabelPattern {
    match policy {
        TimeAxisLabelPolicy::LogicalDecimal { precision } => {
            ResolvedTimeLabelPattern::LogicalDecimal { precision }
        }
        TimeAxisLabelPolicy::UtcDateTime { show_seconds } => {
            let pattern = if show_seconds {
                TimeLabelPattern::DateSecond
            } else {
                TimeLabelPattern::DateMinute
            };
            ResolvedTimeLabelPattern::Utc { pattern }
        }
        TimeAxisLabelPolicy::UtcAdaptive => {
            let pattern = if visible_span_abs <= 600.0 {
                TimeLabelPattern::DateSecond
            } else if visible_span_abs <= 172_800.0 {
                TimeLabelPattern::DateMinute
            } else {
                TimeLabelPattern::Date
            };
            ResolvedTimeLabelPattern::Utc { pattern }
        }
    }
}

fn quantize_logical_time_millis(logical_time: f64) -> i64 {
    if !logical_time.is_finite() {
        return 0;
    }
    let millis = (logical_time * 1_000.0).round();
    if millis > (i64::MAX as f64) {
        i64::MAX
    } else if millis < (i64::MIN as f64) {
        i64::MIN
    } else {
        millis as i64
    }
}

fn quantize_price_label_value(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    let nanos = (value * 1_000_000_000.0).round();
    if nanos > (i64::MAX as f64) {
        i64::MAX
    } else if nanos < (i64::MIN as f64) {
        i64::MIN
    } else {
        nanos as i64
    }
}

fn price_policy_profile(policy: PriceAxisLabelPolicy) -> PriceLabelCachePolicy {
    match policy {
        PriceAxisLabelPolicy::FixedDecimals { precision } => {
            PriceLabelCachePolicy::FixedDecimals { precision }
        }
        PriceAxisLabelPolicy::MinMove {
            min_move,
            trim_trailing_zeros,
        } => PriceLabelCachePolicy::MinMove {
            min_move_nanos: quantize_price_label_value(min_move),
            trim_trailing_zeros,
        },
        PriceAxisLabelPolicy::Adaptive => PriceLabelCachePolicy::Adaptive,
    }
}

fn format_time_axis_label(
    logical_time: f64,
    config: TimeAxisLabelConfig,
    visible_span_abs: f64,
) -> String {
    if !logical_time.is_finite() {
        return "nan".to_owned();
    }

    match resolve_time_label_pattern(config.policy, visible_span_abs) {
        ResolvedTimeLabelPattern::LogicalDecimal { precision } => {
            format_axis_decimal(logical_time, usize::from(precision), config.locale)
        }
        ResolvedTimeLabelPattern::Utc { pattern } => {
            let seconds = logical_time.round() as i64;
            let Some(dt) = DateTime::<Utc>::from_timestamp(seconds, 0) else {
                return format_axis_decimal(logical_time, 2, config.locale);
            };
            let local_dt = dt.with_timezone(&config.timezone.fixed_offset());
            let pattern = resolve_session_time_label_pattern(pattern, config.session, local_dt);

            let pattern = match (config.locale, pattern) {
                (AxisLabelLocale::EnUs, TimeLabelPattern::Date) => "%Y-%m-%d",
                (AxisLabelLocale::EnUs, TimeLabelPattern::DateMinute) => "%Y-%m-%d %H:%M",
                (AxisLabelLocale::EnUs, TimeLabelPattern::DateSecond) => "%Y-%m-%d %H:%M:%S",
                (AxisLabelLocale::EnUs, TimeLabelPattern::TimeMinute) => "%H:%M",
                (AxisLabelLocale::EnUs, TimeLabelPattern::TimeSecond) => "%H:%M:%S",
                (AxisLabelLocale::EsEs, TimeLabelPattern::Date) => "%d/%m/%Y",
                (AxisLabelLocale::EsEs, TimeLabelPattern::DateMinute) => "%d/%m/%Y %H:%M",
                (AxisLabelLocale::EsEs, TimeLabelPattern::DateSecond) => "%d/%m/%Y %H:%M:%S",
                (AxisLabelLocale::EsEs, TimeLabelPattern::TimeMinute) => "%H:%M",
                (AxisLabelLocale::EsEs, TimeLabelPattern::TimeSecond) => "%H:%M:%S",
            };
            local_dt.format(pattern).to_string()
        }
    }
}

fn resolve_session_time_label_pattern(
    pattern: TimeLabelPattern,
    session: Option<TimeAxisSessionConfig>,
    local_dt: DateTime<FixedOffset>,
) -> TimeLabelPattern {
    let Some(session) = session else {
        return pattern;
    };

    // Session mode keeps boundary timestamps explicit while reducing in-session
    // noise to time-only labels for intraday readability.
    let minute_of_day = (local_dt.hour() * 60 + local_dt.minute()) as u16;
    if !session.contains_local_minute(minute_of_day) {
        return pattern;
    }
    if session.is_boundary(minute_of_day, local_dt.second()) {
        return pattern;
    }

    match pattern {
        TimeLabelPattern::DateMinute => TimeLabelPattern::TimeMinute,
        TimeLabelPattern::DateSecond => TimeLabelPattern::TimeSecond,
        other => other,
    }
}

fn is_major_time_tick(logical_time: f64, config: TimeAxisLabelConfig) -> bool {
    if !logical_time.is_finite() {
        return false;
    }
    if matches!(config.policy, TimeAxisLabelPolicy::LogicalDecimal { .. }) {
        return false;
    }

    let seconds = logical_time.round() as i64;
    let Some(dt) = DateTime::<Utc>::from_timestamp(seconds, 0) else {
        return false;
    };
    let local_dt = dt.with_timezone(&config.timezone.fixed_offset());
    let minute_of_day = (local_dt.hour() * 60 + local_dt.minute()) as u16;

    if let Some(session) = config.session {
        if session.is_boundary(minute_of_day, local_dt.second()) {
            return true;
        }
    }

    local_dt.hour() == 0 && local_dt.minute() == 0 && local_dt.second() == 0
}

fn resolved_price_display_base(mode: PriceAxisDisplayMode, fallback_base_price: f64) -> f64 {
    let explicit_base = match mode {
        PriceAxisDisplayMode::Normal => None,
        PriceAxisDisplayMode::Percentage { base_price }
        | PriceAxisDisplayMode::IndexedTo100 { base_price } => base_price,
    };

    let base = explicit_base.unwrap_or(fallback_base_price);
    if !base.is_finite() || base == 0.0 {
        1.0
    } else {
        base
    }
}

fn map_price_to_display_value(
    raw_price: f64,
    mode: PriceAxisDisplayMode,
    fallback_base_price: f64,
) -> f64 {
    if !raw_price.is_finite() {
        return raw_price;
    }

    match mode {
        PriceAxisDisplayMode::Normal => raw_price,
        PriceAxisDisplayMode::Percentage { .. } => {
            let base = resolved_price_display_base(mode, fallback_base_price);
            ((raw_price / base) - 1.0) * 100.0
        }
        PriceAxisDisplayMode::IndexedTo100 { .. } => {
            let base = resolved_price_display_base(mode, fallback_base_price);
            (raw_price / base) * 100.0
        }
    }
}

fn map_price_step_to_display_value(
    raw_step_abs: f64,
    mode: PriceAxisDisplayMode,
    fallback_base_price: f64,
) -> f64 {
    if !raw_step_abs.is_finite() || raw_step_abs <= 0.0 {
        return raw_step_abs;
    }

    match mode {
        PriceAxisDisplayMode::Normal => raw_step_abs,
        PriceAxisDisplayMode::Percentage { .. } | PriceAxisDisplayMode::IndexedTo100 { .. } => {
            let base = resolved_price_display_base(mode, fallback_base_price);
            (raw_step_abs / base).abs() * 100.0
        }
    }
}

fn price_display_mode_suffix(mode: PriceAxisDisplayMode) -> &'static str {
    match mode {
        PriceAxisDisplayMode::Percentage { .. } => "%",
        PriceAxisDisplayMode::Normal | PriceAxisDisplayMode::IndexedTo100 { .. } => "",
    }
}

fn format_price_axis_label(value: f64, config: PriceAxisLabelConfig, tick_step_abs: f64) -> String {
    if !value.is_finite() {
        return "nan".to_owned();
    }

    match config.policy {
        PriceAxisLabelPolicy::FixedDecimals { precision } => {
            format_axis_decimal(value, usize::from(precision), config.locale)
        }
        PriceAxisLabelPolicy::MinMove {
            min_move,
            trim_trailing_zeros,
        } => {
            let precision = precision_from_step(min_move);
            let snapped = if min_move.is_finite() && min_move > 0.0 {
                (value / min_move).round() * min_move
            } else {
                value
            };
            let text = format_axis_decimal(snapped, precision, config.locale);
            if trim_trailing_zeros {
                trim_axis_decimal(text, config.locale)
            } else {
                text
            }
        }
        PriceAxisLabelPolicy::Adaptive => {
            let nice_step = normalize_step_for_precision(tick_step_abs);
            let precision = precision_from_step(nice_step);
            format_axis_decimal(value, precision, config.locale)
        }
    }
}

fn normalize_step_for_precision(step_abs: f64) -> f64 {
    if !step_abs.is_finite() || step_abs <= 0.0 {
        return 0.01;
    }

    let magnitude = 10.0_f64.powf(step_abs.log10().floor());
    if !magnitude.is_finite() || magnitude <= 0.0 {
        return step_abs;
    }

    let normalized = step_abs / magnitude;
    let nice = if normalized < 1.5 {
        1.0
    } else if normalized < 3.0 {
        2.0
    } else if normalized < 7.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

fn precision_from_step(step: f64) -> usize {
    if !step.is_finite() || step <= 0.0 {
        return 2;
    }
    let text = format!("{:.12}", step.abs());
    let Some((_, fraction)) = text.split_once('.') else {
        return 0;
    };
    fraction.trim_end_matches('0').len().clamp(0, 12)
}

fn trim_axis_decimal(mut text: String, locale: AxisLabelLocale) -> String {
    let separator = match locale {
        AxisLabelLocale::EnUs => '.',
        AxisLabelLocale::EsEs => ',',
    };

    if let Some(index) = text.find(separator) {
        let mut trim_start = text.len();
        for (idx, ch) in text.char_indices().rev() {
            if idx <= index {
                break;
            }
            if ch != '0' {
                break;
            }
            trim_start = idx;
        }
        if trim_start < text.len() {
            text.truncate(trim_start);
        }
        if text.ends_with(separator) {
            text.pop();
        }
    }

    if text == "-0" { "0".to_owned() } else { text }
}

fn format_axis_decimal(value: f64, precision: usize, locale: AxisLabelLocale) -> String {
    let text = format!("{value:.precision$}");
    match locale {
        AxisLabelLocale::EnUs => text,
        AxisLabelLocale::EsEs => text.replace('.', ","),
    }
}

const AXIS_TIME_TARGET_SPACING_PX: f64 = 72.0;
const AXIS_TIME_MIN_SPACING_PX: f64 = 56.0;
const AXIS_PRICE_TARGET_SPACING_PX: f64 = 26.0;
const AXIS_PRICE_MIN_SPACING_PX: f64 = 22.0;

fn axis_tick_target_count(
    axis_span_px: f64,
    target_spacing_px: f64,
    min_ticks: usize,
    max_ticks: usize,
) -> usize {
    if !axis_span_px.is_finite() || axis_span_px <= 0.0 {
        return min_ticks;
    }
    if !target_spacing_px.is_finite() || target_spacing_px <= 0.0 {
        return min_ticks;
    }

    let raw = (axis_span_px / target_spacing_px).floor() as usize + 1;
    raw.clamp(min_ticks, max_ticks)
}

fn select_ticks_with_min_spacing(
    mut ticks: Vec<(f64, f64)>,
    min_spacing_px: f64,
) -> Vec<(f64, f64)> {
    if ticks.is_empty() {
        return ticks;
    }

    ticks.sort_by(|left, right| left.1.total_cmp(&right.1));
    if ticks.len() == 1 || !min_spacing_px.is_finite() || min_spacing_px <= 0.0 {
        return ticks;
    }

    let mut selected = Vec::with_capacity(ticks.len());
    selected.push(ticks[0]);

    for tick in ticks.iter().copied().skip(1) {
        if tick.1 - selected.last().expect("not empty").1 >= min_spacing_px {
            selected.push(tick);
        }
    }

    let last_tick = *ticks.last().expect("not empty");
    let selected_last = *selected.last().expect("not empty");
    if selected_last != last_tick {
        if selected.len() == 1 {
            // On very narrow axes a single label is clearer than overlapping pairs.
            selected[0] = last_tick;
        } else {
            let penultimate = selected[selected.len() - 2];
            if last_tick.1 - penultimate.1 >= min_spacing_px {
                let last_index = selected.len() - 1;
                selected[last_index] = last_tick;
            }
        }
    }

    selected
}

fn axis_ticks(range: (f64, f64), tick_count: usize) -> Vec<f64> {
    if tick_count == 0 {
        return Vec::new();
    }

    if tick_count == 1 {
        return vec![range.0];
    }

    let span = range.1 - range.0;
    let denominator = (tick_count - 1) as f64;
    (0..tick_count)
        .map(|index| {
            let ratio = (index as f64) / denominator;
            range.0 + span * ratio
        })
        .collect()
}

fn tick_step_hint_from_values(values: &[f64]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let mut best = f64::INFINITY;
    for pair in values.windows(2) {
        let step = (pair[1] - pair[0]).abs();
        if step.is_finite() && step > 0.0 {
            best = best.min(step);
        }
    }

    if best.is_finite() { best } else { 0.0 }
}

fn validate_kinetic_pan_config(config: KineticPanConfig) -> ChartResult<KineticPanConfig> {
    if !config.decay_per_second.is_finite()
        || config.decay_per_second <= 0.0
        || config.decay_per_second >= 1.0
    {
        return Err(ChartError::InvalidData(
            "kinetic pan decay_per_second must be finite and in (0, 1)".to_owned(),
        ));
    }
    if !config.stop_velocity_abs.is_finite() || config.stop_velocity_abs <= 0.0 {
        return Err(ChartError::InvalidData(
            "kinetic pan stop_velocity_abs must be finite and > 0".to_owned(),
        ));
    }
    Ok(config)
}
