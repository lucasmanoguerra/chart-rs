pub(crate) use crate::extensions::PluginEvent;
pub use crate::interaction::CrosshairMode;

mod render_style;
pub use render_style::{
    CandlestickBodyMode, CrosshairLabelBoxHorizontalAnchor, CrosshairLabelBoxOverflowPolicy,
    CrosshairLabelBoxVerticalAnchor, CrosshairLabelBoxVisibilityPriority,
    CrosshairLabelBoxWidthMode, CrosshairLabelBoxZOrderPolicy, LastPriceLabelBoxWidthMode,
    LastPriceSourceMode, RenderStyle,
};

mod axis_config;
pub use axis_config::{
    AxisLabelLocale, PriceAxisDisplayMode, PriceAxisLabelConfig, PriceAxisLabelPolicy,
    TimeAxisLabelConfig, TimeAxisLabelPolicy, TimeAxisSessionConfig, TimeAxisTimeZone,
};

mod behavior;
pub use behavior::{
    CandlestickBarStyleOverride, CandlestickStyleBehavior, CrosshairAxisLabelBoxStyleBehavior,
    CrosshairAxisLabelStyleBehavior, CrosshairAxisLabelVisibilityBehavior,
    CrosshairGuideLineBehavior, CrosshairGuideLineStyleBehavior, InteractionInputBehavior,
    LastPriceBehavior, PriceScaleMarginBehavior, PriceScaleRealtimeBehavior,
    PriceScaleTransformedBaseBehavior, PriceScaleTransformedBaseSource, StyledOhlcBar,
    TimeCoordinateIndexPolicy, TimeFilledLogicalSlot, TimeFilledLogicalSource,
    TimeScaleEdgeBehavior, TimeScaleNavigationBehavior, TimeScaleRealtimeAppendBehavior,
    TimeScaleResizeAnchor, TimeScaleResizeBehavior, TimeScaleScrollZoomBehavior,
    TimeScaleZoomLimitBehavior,
};

mod label_cache;
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

mod axis_label_format;
mod axis_ticks;
mod chart_behavior;
mod chart_model;
mod chart_presentation;
mod chart_runtime;
mod data_window;
mod engine_config;
mod engine_core;
mod engine_init;
mod engine_snapshot;
mod interaction_validation;
mod invalidation;
mod layout_helpers;
mod lwc_model_sync;

mod axis_adaptive_layout_resolver;
mod axis_adaptive_price_axis_width_resolver;
mod axis_density_coordinator;
mod axis_label_controller;
mod axis_last_price_label_width_estimator;
mod axis_layout_coordinator;
mod axis_layout_pass_resolver;
mod axis_price_axis_relayout_pass_resolver;
mod axis_price_axis_relayout_resolver;
mod axis_price_axis_width_estimator;
mod axis_price_display_context_resolver;
mod axis_price_layout_builder;
mod axis_price_primitives_builder;
mod axis_price_scene_builder;
mod axis_price_tick_exclusion_filter;
mod axis_price_tick_label_width_estimator;
mod axis_price_tick_projection_builder;
mod axis_price_tick_selector;
mod axis_price_tick_spacing_selector;
mod axis_price_width_accumulator;
mod axis_price_width_bounds_resolver;
mod axis_price_width_contribution_accumulator;
mod axis_price_width_contribution_estimator;
mod axis_price_width_display_context_resolver;
mod axis_price_width_display_input_resolver;
mod axis_price_width_pipeline_resolver;
mod axis_price_width_selected_ticks_resolver;
mod axis_price_width_tick_context_resolver;
mod axis_price_width_tick_count_resolver;
mod axis_render_frame_builder;
mod axis_requested_section_sizes_resolver;
mod axis_time_axis_height_estimator;
mod axis_time_scene_builder;
mod cache_profile;
mod candlestick_render_frame_builder;
mod candlestick_style_controller;
mod crosshair_label_box_style_controller;
mod crosshair_label_style_controller;
mod crosshair_label_visibility_controller;
mod crosshair_line_controller;
mod crosshair_line_style_controller;
mod crosshair_render_frame_builder;
mod data_controller;
mod engine_accessors;
mod interaction_controller;
mod interaction_coordinator;
mod label_formatter_controller;
mod label_text_formatter;
mod last_price_axis_label_layout_builder;
mod last_price_axis_label_primitives_builder;
mod last_price_axis_line_primitives_builder;
mod last_price_axis_marker_resolver;
mod last_price_axis_scene_builder;
mod last_price_controller;
mod line_series_render_frame_builder;
mod pane_controller;
mod pane_price_scale_coordinator;
#[cfg(feature = "cairo-backend")]
mod pane_render_executor;
mod pane_scene_coordinator;
mod plugin_dispatch;
mod plugin_registry;
mod price_resolver;
mod price_scale_access;
mod price_scale_coordinator;
mod price_scale_interaction_controller;
mod price_scale_validation;
#[cfg(feature = "cairo-backend")]
mod render_cairo_execution_path_resolver;
#[cfg(feature = "cairo-backend")]
mod render_cairo_partial_input_resolver;
#[cfg(feature = "cairo-backend")]
mod render_cairo_partial_pass_executor;
#[cfg(feature = "cairo-backend")]
mod render_cairo_partial_plan_resolver;
mod render_coordinator;
mod render_frame_builder;
#[cfg(feature = "cairo-backend")]
mod render_partial_lwc_policy_resolver;
#[cfg(feature = "cairo-backend")]
mod render_partial_pane_targets_resolver;
#[cfg(feature = "cairo-backend")]
mod render_partial_plan;
#[cfg(feature = "cairo-backend")]
mod render_partial_plan_pane_targets_resolver;
#[cfg(feature = "cairo-backend")]
mod render_partial_plot_layers_resolver;
#[cfg(feature = "cairo-backend")]
mod render_partial_scheduler;
#[cfg(feature = "cairo-backend")]
mod render_partial_task;
#[cfg(feature = "cairo-backend")]
mod render_partial_task_collectors;
#[cfg(test)]
pub(crate) mod render_partial_test_support;
mod render_style_invalidation_resolver;
mod scale_access;
mod scale_coordinator;
mod series_projection;
mod series_scene_coordinator;
mod snap_resolver;
mod snapshot_controller;
mod time_scale_controller;
mod time_scale_coordinator;
mod time_scale_interaction_controller;
mod time_scale_validation;
mod visible_window_access;

mod engine;
pub use chart_model::ChartModel;
pub(crate) use chart_model::ChartModelBootstrap;
pub use engine::ChartEngine;
pub use engine_config::ChartEngineConfig;
pub use engine_snapshot::{
    CrosshairFormatterDiagnostics, CrosshairFormatterOverrideMode, CrosshairFormatterSnapshot,
    EngineSnapshot,
};

pub use invalidation::{
    InvalidationLevel, InvalidationMask, InvalidationTopic, InvalidationTopics,
    LwcPaneInvalidationSnapshot, LwcPendingInvalidationSnapshot,
};
