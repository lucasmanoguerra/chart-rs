use std::cell::RefCell;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::core::{
    CandleGeometry, DataPoint, OhlcBar, PriceScale, PriceScaleMode, TimeScale, Viewport,
};
use crate::error::{ChartError, ChartResult};
use crate::extensions::{ChartPlugin, PluginEvent};
pub use crate::interaction::CrosshairMode;
use crate::interaction::{CrosshairState, InteractionState};
use crate::render::{Color, LineStrokeStyle, Renderer};

mod render_style;
pub use render_style::{
    CrosshairLabelBoxHorizontalAnchor, CrosshairLabelBoxOverflowPolicy,
    CrosshairLabelBoxVerticalAnchor, CrosshairLabelBoxVisibilityPriority,
    CrosshairLabelBoxWidthMode, CrosshairLabelBoxZOrderPolicy, LastPriceLabelBoxWidthMode,
    LastPriceSourceMode, RenderStyle,
};

mod axis_config;
pub use axis_config::{
    AxisLabelLocale, PriceAxisDisplayMode, PriceAxisLabelConfig, PriceAxisLabelPolicy,
    TimeAxisLabelConfig, TimeAxisLabelPolicy, TimeAxisSessionConfig, TimeAxisTimeZone,
};

mod label_cache;
use label_cache::{PriceLabelCache, TimeLabelCache};
pub use label_cache::{
    PriceLabelCacheStats, PriceLabelFormatterFn, TimeLabelCacheStats, TimeLabelFormatterFn,
};
mod label_formatter_context;
pub use label_formatter_context::{
    CrosshairLabelSourceMode, CrosshairPriceLabelFormatterContext,
    CrosshairPriceLabelFormatterWithContextFn, CrosshairTimeLabelFormatterContext,
    CrosshairTimeLabelFormatterWithContextFn,
};
mod json_contract;
pub use json_contract::{
    CROSSHAIR_DIAGNOSTICS_JSON_SCHEMA_V1, CrosshairFormatterDiagnosticsJsonContractV1,
    ENGINE_SNAPSHOT_JSON_SCHEMA_V1, EngineSnapshotJsonContractV1,
};

mod validation;
use validation::validate_render_style;

mod axis_label_format;
mod axis_ticks;

mod data_window;

mod interaction_validation;

mod layout_helpers;

mod axis_label_controller;
mod cache_profile;
mod crosshair_label_box_style_controller;
mod crosshair_label_style_controller;
mod crosshair_label_visibility_controller;
mod crosshair_line_controller;
mod crosshair_line_style_controller;
mod data_controller;
mod engine_accessors;
mod interaction_controller;
mod label_formatter_controller;
mod last_price_controller;
mod plugin_dispatch;
mod plugin_registry;
mod price_resolver;
mod price_scale_access;
mod price_scale_interaction_controller;
mod render_frame_builder;
mod scale_access;
mod series_projection;
mod snap_resolver;
mod snapshot_controller;
mod time_scale_controller;
mod time_scale_interaction_controller;
mod visible_window_access;

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
    #[serde(default = "default_crosshair_mode")]
    pub crosshair_mode: CrosshairMode,
    #[serde(default = "default_price_scale_mode")]
    pub price_scale_mode: PriceScaleMode,
    #[serde(default)]
    pub price_scale_inverted: bool,
    #[serde(default = "default_price_scale_margins")]
    pub price_scale_margins: PriceScaleMarginBehavior,
    #[serde(default = "default_interaction_input_behavior")]
    pub interaction_input_behavior: InteractionInputBehavior,
    #[serde(default = "default_price_scale_realtime_behavior")]
    pub price_scale_realtime_behavior: PriceScaleRealtimeBehavior,
    #[serde(default = "default_time_scale_navigation_behavior")]
    pub time_scale_navigation_behavior: TimeScaleNavigationBehavior,
    #[serde(default)]
    pub time_scale_right_offset_px: Option<f64>,
    #[serde(default = "default_time_scale_scroll_zoom_behavior")]
    pub time_scale_scroll_zoom_behavior: TimeScaleScrollZoomBehavior,
    #[serde(default = "default_time_scale_zoom_limit_behavior")]
    pub time_scale_zoom_limit_behavior: TimeScaleZoomLimitBehavior,
    #[serde(default = "default_time_scale_edge_behavior")]
    pub time_scale_edge_behavior: TimeScaleEdgeBehavior,
    #[serde(default = "default_time_scale_resize_behavior")]
    pub time_scale_resize_behavior: TimeScaleResizeBehavior,
    #[serde(default = "default_time_scale_realtime_append_behavior")]
    pub time_scale_realtime_append_behavior: TimeScaleRealtimeAppendBehavior,
    #[serde(default = "default_last_price_source_mode")]
    pub last_price_source_mode: LastPriceSourceMode,
    #[serde(default)]
    pub last_price_behavior: Option<LastPriceBehavior>,
    #[serde(default = "default_crosshair_guide_line_behavior")]
    pub crosshair_guide_line_behavior: CrosshairGuideLineBehavior,
    #[serde(default)]
    pub crosshair_guide_line_style_behavior: Option<CrosshairGuideLineStyleBehavior>,
    #[serde(default = "default_crosshair_axis_label_visibility_behavior")]
    pub crosshair_axis_label_visibility_behavior: CrosshairAxisLabelVisibilityBehavior,
    #[serde(default)]
    pub crosshair_axis_label_style_behavior: Option<CrosshairAxisLabelStyleBehavior>,
    #[serde(default)]
    pub crosshair_axis_label_box_style_behavior: Option<CrosshairAxisLabelBoxStyleBehavior>,
    #[serde(default = "default_time_axis_label_config")]
    pub time_axis_label_config: TimeAxisLabelConfig,
    #[serde(default = "default_price_axis_label_config")]
    pub price_axis_label_config: PriceAxisLabelConfig,
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
            crosshair_mode: default_crosshair_mode(),
            price_scale_mode: default_price_scale_mode(),
            price_scale_inverted: false,
            price_scale_margins: default_price_scale_margins(),
            interaction_input_behavior: default_interaction_input_behavior(),
            price_scale_realtime_behavior: default_price_scale_realtime_behavior(),
            time_scale_navigation_behavior: default_time_scale_navigation_behavior(),
            time_scale_right_offset_px: None,
            time_scale_scroll_zoom_behavior: default_time_scale_scroll_zoom_behavior(),
            time_scale_zoom_limit_behavior: default_time_scale_zoom_limit_behavior(),
            time_scale_edge_behavior: default_time_scale_edge_behavior(),
            time_scale_resize_behavior: default_time_scale_resize_behavior(),
            time_scale_realtime_append_behavior: default_time_scale_realtime_append_behavior(),
            last_price_source_mode: default_last_price_source_mode(),
            last_price_behavior: None,
            crosshair_guide_line_behavior: default_crosshair_guide_line_behavior(),
            crosshair_guide_line_style_behavior: None,
            crosshair_axis_label_visibility_behavior:
                default_crosshair_axis_label_visibility_behavior(),
            crosshair_axis_label_style_behavior: None,
            crosshair_axis_label_box_style_behavior: None,
            time_axis_label_config: default_time_axis_label_config(),
            price_axis_label_config: default_price_axis_label_config(),
        }
    }

    /// Sets initial price domain.
    #[must_use]
    pub fn with_price_domain(mut self, price_min: f64, price_max: f64) -> Self {
        self.price_min = price_min;
        self.price_max = price_max;
        self
    }

    /// Sets initial crosshair mode.
    #[must_use]
    pub fn with_crosshair_mode(mut self, mode: CrosshairMode) -> Self {
        self.crosshair_mode = mode;
        self
    }

    /// Sets initial price scale mode.
    #[must_use]
    pub fn with_price_scale_mode(mut self, mode: PriceScaleMode) -> Self {
        self.price_scale_mode = mode;
        self
    }

    /// Sets initial inverted state of price scale mapping.
    #[must_use]
    pub fn with_price_scale_inverted(mut self, inverted: bool) -> Self {
        self.price_scale_inverted = inverted;
        self
    }

    /// Sets initial top/bottom price-scale margins.
    #[must_use]
    pub fn with_price_scale_margins(
        mut self,
        top_margin_ratio: f64,
        bottom_margin_ratio: f64,
    ) -> Self {
        self.price_scale_margins = PriceScaleMarginBehavior {
            top_margin_ratio,
            bottom_margin_ratio,
        };
        self
    }

    /// Sets initial interaction input behavior.
    #[must_use]
    pub fn with_interaction_input_behavior(mut self, behavior: InteractionInputBehavior) -> Self {
        self.interaction_input_behavior = behavior;
        self
    }

    /// Sets initial time-scale navigation behavior.
    #[must_use]
    pub fn with_time_scale_navigation_behavior(
        mut self,
        behavior: TimeScaleNavigationBehavior,
    ) -> Self {
        self.time_scale_navigation_behavior = behavior;
        self
    }

    /// Sets initial time-scale right offset in pixels.
    #[must_use]
    pub fn with_time_scale_right_offset_px(mut self, right_offset_px: Option<f64>) -> Self {
        self.time_scale_right_offset_px = right_offset_px;
        self
    }

    /// Sets initial price-scale realtime behavior.
    #[must_use]
    pub fn with_price_scale_realtime_behavior(
        mut self,
        behavior: PriceScaleRealtimeBehavior,
    ) -> Self {
        self.price_scale_realtime_behavior = behavior;
        self
    }

    /// Sets initial time-scale scroll-zoom behavior.
    #[must_use]
    pub fn with_time_scale_scroll_zoom_behavior(
        mut self,
        behavior: TimeScaleScrollZoomBehavior,
    ) -> Self {
        self.time_scale_scroll_zoom_behavior = behavior;
        self
    }

    /// Sets initial time-scale zoom-limit behavior.
    #[must_use]
    pub fn with_time_scale_zoom_limit_behavior(
        mut self,
        behavior: TimeScaleZoomLimitBehavior,
    ) -> Self {
        self.time_scale_zoom_limit_behavior = behavior;
        self
    }

    /// Sets initial time-scale edge behavior.
    #[must_use]
    pub fn with_time_scale_edge_behavior(mut self, behavior: TimeScaleEdgeBehavior) -> Self {
        self.time_scale_edge_behavior = behavior;
        self
    }

    /// Sets initial time-scale resize behavior.
    #[must_use]
    pub fn with_time_scale_resize_behavior(mut self, behavior: TimeScaleResizeBehavior) -> Self {
        self.time_scale_resize_behavior = behavior;
        self
    }

    /// Sets initial time-scale realtime append behavior.
    #[must_use]
    pub fn with_time_scale_realtime_append_behavior(
        mut self,
        behavior: TimeScaleRealtimeAppendBehavior,
    ) -> Self {
        self.time_scale_realtime_append_behavior = behavior;
        self
    }

    /// Sets initial last-price source mode used by render style.
    #[must_use]
    pub fn with_last_price_source_mode(mut self, mode: LastPriceSourceMode) -> Self {
        self.last_price_source_mode = mode;
        self
    }

    /// Sets initial last-price behavior contract.
    #[must_use]
    pub fn with_last_price_behavior(mut self, behavior: LastPriceBehavior) -> Self {
        self.last_price_behavior = Some(behavior);
        self
    }

    /// Sets initial crosshair guide-line visibility behavior.
    #[must_use]
    pub fn with_crosshair_guide_line_behavior(
        mut self,
        behavior: CrosshairGuideLineBehavior,
    ) -> Self {
        self.crosshair_guide_line_behavior = behavior;
        self
    }

    /// Sets initial crosshair guide-line stroke-style behavior.
    #[must_use]
    pub fn with_crosshair_guide_line_style_behavior(
        mut self,
        behavior: CrosshairGuideLineStyleBehavior,
    ) -> Self {
        self.crosshair_guide_line_style_behavior = Some(behavior);
        self
    }

    /// Sets initial crosshair axis-label visibility behavior.
    #[must_use]
    pub fn with_crosshair_axis_label_visibility_behavior(
        mut self,
        behavior: CrosshairAxisLabelVisibilityBehavior,
    ) -> Self {
        self.crosshair_axis_label_visibility_behavior = behavior;
        self
    }

    /// Sets initial crosshair axis-label style behavior.
    #[must_use]
    pub fn with_crosshair_axis_label_style_behavior(
        mut self,
        behavior: CrosshairAxisLabelStyleBehavior,
    ) -> Self {
        self.crosshair_axis_label_style_behavior = Some(behavior);
        self
    }

    /// Sets initial crosshair axis-label box style behavior.
    #[must_use]
    pub fn with_crosshair_axis_label_box_style_behavior(
        mut self,
        behavior: CrosshairAxisLabelBoxStyleBehavior,
    ) -> Self {
        self.crosshair_axis_label_box_style_behavior = Some(behavior);
        self
    }

    /// Sets initial time-axis label formatter config.
    #[must_use]
    pub fn with_time_axis_label_config(mut self, config: TimeAxisLabelConfig) -> Self {
        self.time_axis_label_config = config;
        self
    }

    /// Sets initial price-axis label formatter config.
    #[must_use]
    pub fn with_price_axis_label_config(mut self, config: PriceAxisLabelConfig) -> Self {
        self.price_axis_label_config = config;
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

fn default_crosshair_mode() -> CrosshairMode {
    CrosshairMode::Magnet
}

fn default_price_scale_mode() -> PriceScaleMode {
    PriceScaleMode::Linear
}

fn default_price_scale_margins() -> PriceScaleMarginBehavior {
    PriceScaleMarginBehavior::default()
}

fn default_interaction_input_behavior() -> InteractionInputBehavior {
    InteractionInputBehavior::default()
}

fn default_time_scale_navigation_behavior() -> TimeScaleNavigationBehavior {
    TimeScaleNavigationBehavior::default()
}

fn default_price_scale_realtime_behavior() -> PriceScaleRealtimeBehavior {
    PriceScaleRealtimeBehavior::default()
}

fn default_time_scale_scroll_zoom_behavior() -> TimeScaleScrollZoomBehavior {
    TimeScaleScrollZoomBehavior::default()
}

fn default_time_scale_zoom_limit_behavior() -> TimeScaleZoomLimitBehavior {
    TimeScaleZoomLimitBehavior::default()
}

fn default_time_scale_edge_behavior() -> TimeScaleEdgeBehavior {
    TimeScaleEdgeBehavior::default()
}

fn default_time_scale_resize_behavior() -> TimeScaleResizeBehavior {
    TimeScaleResizeBehavior::default()
}

fn default_time_scale_realtime_append_behavior() -> TimeScaleRealtimeAppendBehavior {
    TimeScaleRealtimeAppendBehavior::default()
}

fn default_last_price_source_mode() -> LastPriceSourceMode {
    LastPriceSourceMode::default()
}

fn default_crosshair_guide_line_behavior() -> CrosshairGuideLineBehavior {
    CrosshairGuideLineBehavior::default()
}

fn default_crosshair_axis_label_visibility_behavior() -> CrosshairAxisLabelVisibilityBehavior {
    CrosshairAxisLabelVisibilityBehavior::default()
}

fn default_time_axis_label_config() -> TimeAxisLabelConfig {
    TimeAxisLabelConfig::default()
}

fn default_price_axis_label_config() -> PriceAxisLabelConfig {
    PriceAxisLabelConfig::default()
}

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
}

impl Default for PriceScaleRealtimeBehavior {
    fn default() -> Self {
        Self {
            autoscale_on_data_set: true,
            autoscale_on_data_update: true,
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

/// Serializable deterministic state snapshot used by regression tests and
/// debugging tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrosshairFormatterOverrideMode {
    None,
    Legacy,
    Context,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosshairFormatterSnapshot {
    pub time_override_mode: CrosshairFormatterOverrideMode,
    pub price_override_mode: CrosshairFormatterOverrideMode,
    pub time_formatter_generation: u64,
    pub price_formatter_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosshairFormatterDiagnostics {
    pub time_override_mode: CrosshairFormatterOverrideMode,
    pub price_override_mode: CrosshairFormatterOverrideMode,
    pub time_formatter_generation: u64,
    pub price_formatter_generation: u64,
    pub time_cache: TimeLabelCacheStats,
    pub price_cache: PriceLabelCacheStats,
}

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
    pub crosshair_formatter: CrosshairFormatterSnapshot,
}

/// Main orchestration facade consumed by host applications.
///
/// `ChartEngine` coordinates time/price scales, interaction state,
/// data/candle collections, and renderer calls.
pub struct ChartEngine<R: Renderer> {
    renderer: R,
    viewport: Viewport,
    time_scale: TimeScale,
    time_scale_edge_behavior: TimeScaleEdgeBehavior,
    time_scale_navigation_behavior: TimeScaleNavigationBehavior,
    time_scale_zoom_limit_behavior: TimeScaleZoomLimitBehavior,
    time_scale_right_offset_px: Option<f64>,
    time_scale_scroll_zoom_behavior: TimeScaleScrollZoomBehavior,
    time_scale_resize_behavior: TimeScaleResizeBehavior,
    time_scale_realtime_append_behavior: TimeScaleRealtimeAppendBehavior,
    price_scale_realtime_behavior: PriceScaleRealtimeBehavior,
    interaction_input_behavior: InteractionInputBehavior,
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
    crosshair_time_label_formatter: Option<TimeLabelFormatterFn>,
    crosshair_price_label_formatter: Option<PriceLabelFormatterFn>,
    crosshair_time_label_formatter_with_context: Option<CrosshairTimeLabelFormatterWithContextFn>,
    crosshair_price_label_formatter_with_context: Option<CrosshairPriceLabelFormatterWithContextFn>,
    time_label_formatter_generation: u64,
    price_label_formatter_generation: u64,
    crosshair_time_label_formatter_generation: u64,
    crosshair_price_label_formatter_generation: u64,
    time_label_cache: RefCell<TimeLabelCache>,
    price_label_cache: RefCell<PriceLabelCache>,
    crosshair_time_label_cache: RefCell<TimeLabelCache>,
    crosshair_price_label_cache: RefCell<PriceLabelCache>,
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
        let price_scale =
            PriceScale::new_with_mode(config.price_min, config.price_max, config.price_scale_mode)?
                .with_inverted(config.price_scale_inverted)
                .with_margins(
                    config.price_scale_margins.top_margin_ratio,
                    config.price_scale_margins.bottom_margin_ratio,
                )?;
        let mut interaction = InteractionState::default();
        interaction.set_crosshair_mode(config.crosshair_mode);

        let mut engine = Self {
            renderer,
            viewport: config.viewport,
            time_scale,
            time_scale_edge_behavior: TimeScaleEdgeBehavior::default(),
            time_scale_navigation_behavior: TimeScaleNavigationBehavior::default(),
            time_scale_zoom_limit_behavior: TimeScaleZoomLimitBehavior::default(),
            time_scale_right_offset_px: None,
            time_scale_scroll_zoom_behavior: TimeScaleScrollZoomBehavior::default(),
            time_scale_resize_behavior: TimeScaleResizeBehavior::default(),
            time_scale_realtime_append_behavior: TimeScaleRealtimeAppendBehavior::default(),
            price_scale_realtime_behavior: config.price_scale_realtime_behavior,
            interaction_input_behavior: config.interaction_input_behavior,
            price_scale,
            price_scale_mode: config.price_scale_mode,
            interaction,
            points: Vec::new(),
            candles: Vec::new(),
            series_metadata: IndexMap::new(),
            plugins: Vec::new(),
            time_axis_label_config: TimeAxisLabelConfig::default(),
            price_axis_label_config: PriceAxisLabelConfig::default(),
            time_label_formatter: None,
            price_label_formatter: None,
            crosshair_time_label_formatter: None,
            crosshair_price_label_formatter: None,
            crosshair_time_label_formatter_with_context: None,
            crosshair_price_label_formatter_with_context: None,
            time_label_formatter_generation: 0,
            price_label_formatter_generation: 0,
            crosshair_time_label_formatter_generation: 0,
            crosshair_price_label_formatter_generation: 0,
            time_label_cache: RefCell::new(TimeLabelCache::default()),
            price_label_cache: RefCell::new(PriceLabelCache::default()),
            crosshair_time_label_cache: RefCell::new(TimeLabelCache::default()),
            crosshair_price_label_cache: RefCell::new(PriceLabelCache::default()),
            render_style: RenderStyle::default(),
        };

        if config.time_scale_navigation_behavior != TimeScaleNavigationBehavior::default() {
            engine.set_time_scale_navigation_behavior(config.time_scale_navigation_behavior)?;
        }
        if config.time_scale_right_offset_px.is_some() {
            engine.set_time_scale_right_offset_px(config.time_scale_right_offset_px)?;
        }
        if config.time_scale_scroll_zoom_behavior != TimeScaleScrollZoomBehavior::default() {
            engine.set_time_scale_scroll_zoom_behavior(config.time_scale_scroll_zoom_behavior)?;
        }
        if config.time_scale_zoom_limit_behavior != TimeScaleZoomLimitBehavior::default() {
            engine.set_time_scale_zoom_limit_behavior(config.time_scale_zoom_limit_behavior)?;
        }
        if config.time_scale_edge_behavior != TimeScaleEdgeBehavior::default() {
            engine.set_time_scale_edge_behavior(config.time_scale_edge_behavior)?;
        }
        if config.time_scale_resize_behavior != TimeScaleResizeBehavior::default() {
            engine.set_time_scale_resize_behavior(config.time_scale_resize_behavior)?;
        }
        if config.time_scale_realtime_append_behavior != TimeScaleRealtimeAppendBehavior::default()
        {
            engine.set_time_scale_realtime_append_behavior(
                config.time_scale_realtime_append_behavior,
            )?;
        }
        if config.last_price_source_mode != LastPriceSourceMode::default() {
            let mut style = engine.render_style();
            style.last_price_source_mode = config.last_price_source_mode;
            engine.set_render_style(style)?;
        }
        if let Some(behavior) = config.last_price_behavior {
            engine.set_last_price_behavior(behavior)?;
        }
        if config.crosshair_guide_line_behavior != CrosshairGuideLineBehavior::default() {
            engine.set_crosshair_guide_line_behavior(config.crosshair_guide_line_behavior)?;
        }
        if let Some(behavior) = config.crosshair_guide_line_style_behavior {
            engine.set_crosshair_guide_line_style_behavior(behavior)?;
        }
        if config.crosshair_axis_label_visibility_behavior
            != CrosshairAxisLabelVisibilityBehavior::default()
        {
            engine.set_crosshair_axis_label_visibility_behavior(
                config.crosshair_axis_label_visibility_behavior,
            )?;
        }
        if let Some(behavior) = config.crosshair_axis_label_style_behavior {
            engine.set_crosshair_axis_label_style_behavior(behavior)?;
        }
        if let Some(behavior) = config.crosshair_axis_label_box_style_behavior {
            engine.set_crosshair_axis_label_box_style_behavior(behavior)?;
        }
        engine.set_time_axis_label_config(config.time_axis_label_config)?;
        engine.set_price_axis_label_config(config.price_axis_label_config)?;

        Ok(engine)
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
}
