use super::{
    InteractionInputBehavior, PriceAxisLabelConfig, PriceScaleRealtimeBehavior,
    PriceScaleTransformedBaseBehavior, TimeAxisLabelConfig, TimeScaleEdgeBehavior,
    TimeScaleNavigationBehavior, TimeScaleRealtimeAppendBehavior, TimeScaleResizeBehavior,
    TimeScaleScrollZoomBehavior, TimeScaleZoomLimitBehavior,
};

/// Runtime behavior/configuration state grouped separately from core chart data.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct ChartBehaviorState {
    pub(super) time_scale_edge_behavior: TimeScaleEdgeBehavior,
    pub(super) time_scale_navigation_behavior: TimeScaleNavigationBehavior,
    pub(super) time_scale_zoom_limit_behavior: TimeScaleZoomLimitBehavior,
    pub(super) time_scale_right_offset_px: Option<f64>,
    pub(super) time_scale_scroll_zoom_behavior: TimeScaleScrollZoomBehavior,
    pub(super) time_scale_resize_behavior: TimeScaleResizeBehavior,
    pub(super) time_scale_realtime_append_behavior: TimeScaleRealtimeAppendBehavior,
    pub(super) price_scale_realtime_behavior: PriceScaleRealtimeBehavior,
    pub(super) interaction_input_behavior: InteractionInputBehavior,
    pub(super) price_scale_transformed_base_behavior: PriceScaleTransformedBaseBehavior,
    pub(super) time_axis_label_config: TimeAxisLabelConfig,
    pub(super) price_axis_label_config: PriceAxisLabelConfig,
}
