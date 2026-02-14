use serde::{Deserialize, Serialize};

use crate::core::OhlcBar;
use crate::error::ChartResult;
use crate::render::{Color, LineStrokeStyle};

use super::{CandlestickBodyMode, LastPriceSourceMode, RenderStyle};

fn default_true() -> bool {
    true
}

/// Time-scale edge constraint policy for visible-range navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TimeScaleEdgeBehavior {
    pub fix_left_edge: bool,
    pub fix_right_edge: bool,
}

/// Host-configurable interaction input gates aligned with Lightweight Charts
/// `handleScroll` / `handleScale` behavior families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionInputBehavior {
    /// Master enable for scroll-family interactions.
    pub handle_scroll: bool,
    /// Master enable for scale-family interactions.
    pub handle_scale: bool,
    /// Enables wheel-driven horizontal scroll/pan.
    pub scroll_mouse_wheel: bool,
    /// Enables pressed-mouse drag scrolling.
    pub scroll_pressed_mouse_move: bool,
    /// Reserved parity knob for horizontal touch drag scrolling.
    pub scroll_horz_touch_drag: bool,
    /// Reserved parity knob for vertical touch drag scrolling.
    pub scroll_vert_touch_drag: bool,
    /// Enables wheel-driven zoom.
    pub scale_mouse_wheel: bool,
    /// Enables pinch-driven zoom.
    pub scale_pinch: bool,
    /// Enables axis drag-to-scale behavior.
    #[serde(default = "default_true")]
    pub scale_axis_pressed_mouse_move: bool,
    /// Enables axis double-click reset behavior.
    #[serde(default = "default_true")]
    pub scale_axis_double_click_reset: bool,
}

impl Default for InteractionInputBehavior {
    fn default() -> Self {
        Self {
            handle_scroll: true,
            handle_scale: true,
            scroll_mouse_wheel: true,
            scroll_pressed_mouse_move: true,
            scroll_horz_touch_drag: true,
            scroll_vert_touch_drag: true,
            scale_mouse_wheel: true,
            scale_pinch: true,
            scale_axis_pressed_mouse_move: true,
            scale_axis_double_click_reset: true,
        }
    }
}

impl InteractionInputBehavior {
    #[must_use]
    pub(crate) fn allows_drag_pan(self) -> bool {
        self.handle_scroll && self.scroll_pressed_mouse_move
    }

    #[must_use]
    pub(crate) fn allows_wheel_pan(self) -> bool {
        self.handle_scroll && self.scroll_mouse_wheel
    }

    #[must_use]
    pub(crate) fn allows_wheel_zoom(self) -> bool {
        self.handle_scale && self.scale_mouse_wheel
    }

    #[must_use]
    pub(crate) fn allows_pinch_zoom(self) -> bool {
        self.handle_scale && self.scale_pinch
    }

    #[must_use]
    pub(crate) fn allows_axis_drag_scale(self) -> bool {
        self.handle_scale && self.scale_axis_pressed_mouse_move
    }

    #[must_use]
    pub(crate) fn allows_axis_double_click_reset(self) -> bool {
        self.handle_scale && self.scale_axis_double_click_reset
    }
}

/// Time-scale navigation behavior aligned with Lightweight Charts style
/// right-offset and spacing controls.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeScaleNavigationBehavior {
    /// Logical bar offset from the latest data bar at the right edge.
    ///
    /// Positive values keep extra whitespace on the right side.
    pub right_offset_bars: f64,
    /// Optional target bar spacing in pixels.
    ///
    /// `None` preserves current visible span.
    pub bar_spacing_px: Option<f64>,
}

impl Default for TimeScaleNavigationBehavior {
    fn default() -> Self {
        Self {
            right_offset_bars: 0.0,
            bar_spacing_px: Some(6.0),
        }
    }
}

/// Coordinate-to-logical-index mapping policy for sparse time series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeCoordinateIndexPolicy {
    /// Returns floating logical indices, including whitespace slots.
    #[default]
    AllowWhitespace,
    /// Returns nearest filled logical index, skipping whitespace slots.
    IgnoreWhitespace,
}

/// Source collection of a resolved filled logical slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeFilledLogicalSource {
    Points,
    Candles,
}

/// Filled logical-slot descriptor for sparse-series host integrations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeFilledLogicalSlot {
    pub source: TimeFilledLogicalSource,
    pub slot: usize,
    pub logical_index: f64,
    pub time: f64,
}

/// Time-scale zoom limits derived from effective bar spacing in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeScaleZoomLimitBehavior {
    /// Minimum allowed spacing between bars in pixels (zoom-out limit).
    pub min_bar_spacing_px: f64,
    /// Optional maximum allowed spacing between bars in pixels (zoom-in limit).
    pub max_bar_spacing_px: Option<f64>,
}

impl Default for TimeScaleZoomLimitBehavior {
    fn default() -> Self {
        Self {
            min_bar_spacing_px: 0.5,
            max_bar_spacing_px: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeScaleResizeAnchor {
    Left,
    Center,
    Right,
}

/// Time-scale policy used when viewport width changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeScaleResizeBehavior {
    pub lock_visible_range_on_resize: bool,
    pub anchor: TimeScaleResizeAnchor,
}

impl Default for TimeScaleResizeBehavior {
    fn default() -> Self {
        Self {
            lock_visible_range_on_resize: false,
            anchor: TimeScaleResizeAnchor::Right,
        }
    }
}

/// Time-scale policy for scroll-zoom anchoring semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TimeScaleScrollZoomBehavior {
    /// Keeps right edge anchored during wheel/pinch zoom when enabled.
    pub right_bar_stays_on_scroll: bool,
}

/// Time-scale behavior for realtime append flows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeScaleRealtimeAppendBehavior {
    pub preserve_right_edge_on_append: bool,
    /// Right-edge tracking tolerance expressed in reference bars.
    pub right_edge_tolerance_bars: f64,
}

impl Default for TimeScaleRealtimeAppendBehavior {
    fn default() -> Self {
        Self {
            preserve_right_edge_on_append: true,
            right_edge_tolerance_bars: 0.75,
        }
    }
}

/// Price-scale behavior for realtime data-update flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceScaleRealtimeBehavior {
    /// Enables best-effort autoscale after full data replacement (`set_*`).
    pub autoscale_on_data_set: bool,
    /// Enables best-effort autoscale after append/update data mutations.
    pub autoscale_on_data_update: bool,
    /// Enables best-effort autoscale from visible window after time-range
    /// navigation updates (pan/zoom/fit/scroll/resize policies).
    #[serde(default = "default_true")]
    pub autoscale_on_time_range_change: bool,
}

impl Default for PriceScaleRealtimeBehavior {
    fn default() -> Self {
        Self {
            autoscale_on_data_set: true,
            autoscale_on_data_update: true,
            autoscale_on_time_range_change: true,
        }
    }
}

/// Dynamic source used when resolving transformed base for
/// `PriceScaleMode::{Percentage, IndexedTo100}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PriceScaleTransformedBaseSource {
    /// Uses current scale domain start as transformed base.
    #[default]
    DomainStart,
    /// Uses earliest loaded sample by time (point/candle close).
    FirstData,
    /// Uses latest loaded sample by time (point/candle close).
    LastData,
    /// Uses earliest visible sample by time.
    FirstVisibleData,
    /// Uses latest visible sample by time.
    LastVisibleData,
}

/// Base-resolution policy for transformed price-scale modes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PriceScaleTransformedBaseBehavior {
    /// Optional explicit base override; when set, has highest priority.
    pub explicit_base_price: Option<f64>,
    /// Dynamic fallback source when explicit base is not set.
    pub dynamic_source: PriceScaleTransformedBaseSource,
}

impl Default for PriceScaleTransformedBaseBehavior {
    fn default() -> Self {
        Self {
            explicit_base_price: None,
            dynamic_source: PriceScaleTransformedBaseSource::DomainStart,
        }
    }
}

/// Price-scale margin behavior (top/bottom whitespace ratios).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PriceScaleMarginBehavior {
    pub top_margin_ratio: f64,
    pub bottom_margin_ratio: f64,
}

impl Default for PriceScaleMarginBehavior {
    fn default() -> Self {
        Self {
            top_margin_ratio: 0.2,
            bottom_margin_ratio: 0.1,
        }
    }
}

/// Crosshair guide-line visibility behavior (`shared && axis`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosshairGuideLineBehavior {
    pub show_lines: bool,
    pub show_horizontal_line: bool,
    pub show_vertical_line: bool,
}

impl Default for CrosshairGuideLineBehavior {
    fn default() -> Self {
        Self {
            show_lines: true,
            show_horizontal_line: true,
            show_vertical_line: true,
        }
    }
}

/// Crosshair guide-line stroke-style behavior (shared + per-axis overrides).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrosshairGuideLineStyleBehavior {
    pub line_color: Color,
    pub line_width: f64,
    pub line_style: LineStrokeStyle,
    pub horizontal_line_color: Option<Color>,
    pub horizontal_line_width: Option<f64>,
    pub horizontal_line_style: Option<LineStrokeStyle>,
    pub vertical_line_color: Option<Color>,
    pub vertical_line_width: Option<f64>,
    pub vertical_line_style: Option<LineStrokeStyle>,
}

impl Default for CrosshairGuideLineStyleBehavior {
    fn default() -> Self {
        let style = RenderStyle::default();
        Self {
            line_color: style.crosshair_line_color,
            line_width: style.crosshair_line_width,
            line_style: style.crosshair_line_style,
            horizontal_line_color: style.crosshair_horizontal_line_color,
            horizontal_line_width: style.crosshair_horizontal_line_width,
            horizontal_line_style: style.crosshair_horizontal_line_style,
            vertical_line_color: style.crosshair_vertical_line_color,
            vertical_line_width: style.crosshair_vertical_line_width,
            vertical_line_style: style.crosshair_vertical_line_style,
        }
    }
}

/// Crosshair axis-label visibility behavior (labels, boxes, and box borders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosshairAxisLabelVisibilityBehavior {
    pub show_time_label: bool,
    pub show_price_label: bool,
    pub show_time_label_box: bool,
    pub show_price_label_box: bool,
    pub show_time_label_box_border: bool,
    pub show_price_label_box_border: bool,
}

impl Default for CrosshairAxisLabelVisibilityBehavior {
    fn default() -> Self {
        Self {
            show_time_label: true,
            show_price_label: true,
            show_time_label_box: true,
            show_price_label_box: true,
            show_time_label_box_border: true,
            show_price_label_box_border: true,
        }
    }
}

/// Crosshair axis-label style behavior (colors, font sizes, offsets, and insets).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrosshairAxisLabelStyleBehavior {
    pub time_label_color: Color,
    pub price_label_color: Color,
    pub time_label_font_size_px: f64,
    pub price_label_font_size_px: f64,
    pub time_label_offset_y_px: f64,
    pub price_label_offset_y_px: f64,
    pub time_label_padding_x_px: f64,
    pub price_label_padding_right_px: f64,
}

impl Default for CrosshairAxisLabelStyleBehavior {
    fn default() -> Self {
        let style = RenderStyle::default();
        Self {
            time_label_color: style.crosshair_time_label_color,
            price_label_color: style.crosshair_price_label_color,
            time_label_font_size_px: style.crosshair_time_label_font_size_px,
            price_label_font_size_px: style.crosshair_price_label_font_size_px,
            time_label_offset_y_px: style.crosshair_time_label_offset_y_px,
            price_label_offset_y_px: style.crosshair_price_label_offset_y_px,
            time_label_padding_x_px: style.crosshair_time_label_padding_x_px,
            price_label_padding_right_px: style.crosshair_price_label_padding_right_px,
        }
    }
}

/// Crosshair axis-label box style behavior (shared + per-axis fill/border/radius).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrosshairAxisLabelBoxStyleBehavior {
    pub box_color: Color,
    pub time_box_color: Option<Color>,
    pub price_box_color: Option<Color>,
    pub box_border_color: Color,
    pub time_box_border_color: Color,
    pub price_box_border_color: Color,
    pub box_border_width_px: f64,
    pub time_box_border_width_px: f64,
    pub price_box_border_width_px: f64,
    pub box_corner_radius_px: f64,
    pub time_box_corner_radius_px: f64,
    pub price_box_corner_radius_px: f64,
}

impl Default for CrosshairAxisLabelBoxStyleBehavior {
    fn default() -> Self {
        let style = RenderStyle::default();
        Self {
            box_color: style.crosshair_label_box_color,
            time_box_color: style.crosshair_time_label_box_color,
            price_box_color: style.crosshair_price_label_box_color,
            box_border_color: style.crosshair_label_box_border_color,
            time_box_border_color: style.crosshair_time_label_box_border_color,
            price_box_border_color: style.crosshair_price_label_box_border_color,
            box_border_width_px: style.crosshair_label_box_border_width_px,
            time_box_border_width_px: style.crosshair_time_label_box_border_width_px,
            price_box_border_width_px: style.crosshair_price_label_box_border_width_px,
            box_corner_radius_px: style.crosshair_label_box_corner_radius_px,
            time_box_corner_radius_px: style.crosshair_time_label_box_corner_radius_px,
            price_box_corner_radius_px: style.crosshair_price_label_box_corner_radius_px,
        }
    }
}

/// Last-price marker behavior contract (line/label/source/trend-color policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastPriceBehavior {
    pub show_line: bool,
    pub show_label: bool,
    pub use_trend_color: bool,
    pub source_mode: LastPriceSourceMode,
}

impl Default for LastPriceBehavior {
    fn default() -> Self {
        Self {
            show_line: true,
            show_label: true,
            use_trend_color: false,
            source_mode: LastPriceSourceMode::LatestData,
        }
    }
}

/// Candlestick style behavior contract (colors, body policy, and visibility/stroke controls).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CandlestickStyleBehavior {
    pub up_color: Color,
    pub down_color: Color,
    /// Shared wick color override (Lightweight `wickColor` parity).
    /// When set, it overrides `wick_up_color` and `wick_down_color`.
    pub wick_color: Option<Color>,
    pub wick_up_color: Color,
    pub wick_down_color: Color,
    /// Shared border color override (Lightweight `borderColor` parity).
    /// When set, it overrides `border_up_color` and `border_down_color`.
    pub border_color: Option<Color>,
    pub border_up_color: Color,
    pub border_down_color: Color,
    pub body_mode: CandlestickBodyMode,
    pub wick_width_px: f64,
    pub border_width_px: f64,
    pub show_wicks: bool,
    pub show_borders: bool,
}

impl Default for CandlestickStyleBehavior {
    fn default() -> Self {
        let style = RenderStyle::default();
        Self {
            up_color: style.candlestick_up_color,
            down_color: style.candlestick_down_color,
            wick_color: None,
            wick_up_color: style.candlestick_wick_up_color,
            wick_down_color: style.candlestick_wick_down_color,
            border_color: None,
            border_up_color: style.candlestick_border_up_color,
            border_down_color: style.candlestick_border_down_color,
            body_mode: style.candlestick_body_mode,
            wick_width_px: style.candlestick_wick_width_px,
            border_width_px: style.candlestick_border_width_px,
            show_wicks: style.show_candlestick_wicks,
            show_borders: style.show_candlestick_borders,
        }
    }
}

/// Optional per-bar candlestick color overrides.
///
/// Equivalent to Lightweight bar-level `color`, `wickColor`, and `borderColor`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CandlestickBarStyleOverride {
    pub color: Option<Color>,
    pub wick_color: Option<Color>,
    pub border_color: Option<Color>,
}

impl CandlestickBarStyleOverride {
    pub fn validate(self) -> ChartResult<()> {
        if let Some(color) = self.color {
            color.validate()?;
        }
        if let Some(color) = self.wick_color {
            color.validate()?;
        }
        if let Some(color) = self.border_color {
            color.validate()?;
        }
        Ok(())
    }
}

/// OHLC bar payload with optional per-bar candlestick style override.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StyledOhlcBar {
    pub ohlc: OhlcBar,
    pub style_override: Option<CandlestickBarStyleOverride>,
}

impl StyledOhlcBar {
    #[must_use]
    pub const fn new(ohlc: OhlcBar) -> Self {
        Self {
            ohlc,
            style_override: None,
        }
    }

    #[must_use]
    pub const fn with_style_override(
        mut self,
        style_override: CandlestickBarStyleOverride,
    ) -> Self {
        self.style_override = Some(style_override);
        self
    }
}
