use serde::{Deserialize, Serialize};

use crate::core::{PriceScaleMode, Viewport};
use crate::error::{ChartError, ChartResult};
use crate::interaction::CrosshairMode;

use super::{
    CandlestickStyleBehavior, CrosshairAxisLabelBoxStyleBehavior, CrosshairAxisLabelStyleBehavior,
    CrosshairAxisLabelVisibilityBehavior, CrosshairGuideLineBehavior,
    CrosshairGuideLineStyleBehavior, InteractionInputBehavior, LastPriceBehavior,
    LastPriceSourceMode, PriceAxisLabelConfig, PriceScaleMarginBehavior,
    PriceScaleRealtimeBehavior, PriceScaleTransformedBaseBehavior, TimeAxisLabelConfig,
    TimeScaleEdgeBehavior, TimeScaleNavigationBehavior, TimeScaleRealtimeAppendBehavior,
    TimeScaleResizeBehavior, TimeScaleScrollZoomBehavior, TimeScaleZoomLimitBehavior,
};

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
    #[serde(default = "default_price_scale_transformed_base_behavior")]
    pub price_scale_transformed_base_behavior: PriceScaleTransformedBaseBehavior,
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
    #[serde(default)]
    pub candlestick_style_behavior: Option<CandlestickStyleBehavior>,
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
            price_scale_transformed_base_behavior: default_price_scale_transformed_base_behavior(),
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
            candlestick_style_behavior: None,
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

    /// Sets transformed base behavior for percentage/indexed price-scale modes.
    #[must_use]
    pub fn with_price_scale_transformed_base_behavior(
        mut self,
        behavior: PriceScaleTransformedBaseBehavior,
    ) -> Self {
        self.price_scale_transformed_base_behavior = behavior;
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

    /// Sets initial candlestick style behavior contract.
    #[must_use]
    pub fn with_candlestick_style_behavior(mut self, behavior: CandlestickStyleBehavior) -> Self {
        self.candlestick_style_behavior = Some(behavior);
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

fn default_price_scale_transformed_base_behavior() -> PriceScaleTransformedBaseBehavior {
    PriceScaleTransformedBaseBehavior::default()
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
