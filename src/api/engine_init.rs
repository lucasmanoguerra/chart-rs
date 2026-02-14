use crate::core::{PaneCollection, PriceScale, TimeScale};
use crate::error::{ChartError, ChartResult};
use crate::interaction::InteractionState;
use crate::render::Renderer;

use super::{
    ChartEngine, ChartEngineConfig, ChartModel, ChartModelBootstrap,
    CrosshairAxisLabelVisibilityBehavior, CrosshairGuideLineBehavior, LastPriceSourceMode,
    TimeScaleEdgeBehavior, TimeScaleNavigationBehavior, TimeScaleRealtimeAppendBehavior,
    TimeScaleResizeBehavior, TimeScaleScrollZoomBehavior, TimeScaleZoomLimitBehavior,
    chart_behavior::ChartBehaviorState, chart_presentation::ChartPresentationState,
    chart_runtime::ChartRuntimeState, engine_core::EngineCore,
};

impl<R: Renderer> ChartEngine<R> {
    /// Creates a fully initialized engine with explicit domains.
    pub fn new(renderer: R, config: ChartEngineConfig) -> ChartResult<Self> {
        if !config.viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: config.viewport.width,
                height: config.viewport.height,
            });
        }
        if let Some(explicit_base) = config
            .price_scale_transformed_base_behavior
            .explicit_base_price
        {
            if !explicit_base.is_finite() || explicit_base == 0.0 {
                return Err(ChartError::InvalidData(
                    "price scale transformed explicit base must be finite and non-zero".to_owned(),
                ));
            }
        }

        let time_scale = TimeScale::new(config.time_start, config.time_end)?;
        let price_scale = PriceScale::new_with_mode_and_base(
            config.price_min,
            config.price_max,
            config.price_scale_mode,
            config
                .price_scale_transformed_base_behavior
                .explicit_base_price,
        )?
        .with_inverted(config.price_scale_inverted)
        .with_margins(
            config.price_scale_margins.top_margin_ratio,
            config.price_scale_margins.bottom_margin_ratio,
        )?;
        let mut interaction = InteractionState::default();
        interaction.set_crosshair_mode(config.crosshair_mode);
        let pane_collection = PaneCollection::default();
        let main_pane_id = pane_collection.main_pane_id();
        let model = ChartModel::new(ChartModelBootstrap {
            viewport: config.viewport,
            time_scale,
            price_scale,
            price_scale_mode: config.price_scale_mode,
            interaction,
            pane_collection,
            points_pane_id: main_pane_id,
            candles_pane_id: main_pane_id,
        });

        let mut engine = Self {
            renderer,
            core: EngineCore {
                model,
                lwc_model: crate::lwc::model::ChartModel::with_default_pane(
                    config.viewport.width as f64,
                ),
                behavior: ChartBehaviorState {
                    price_scale_realtime_behavior: config.price_scale_realtime_behavior,
                    interaction_input_behavior: config.interaction_input_behavior,
                    price_scale_transformed_base_behavior: config
                        .price_scale_transformed_base_behavior,
                    ..ChartBehaviorState::default()
                },
                presentation: ChartPresentationState::default(),
                runtime: ChartRuntimeState::with_full_invalidation(),
            },
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
        if let Some(behavior) = config.candlestick_style_behavior {
            engine.set_candlestick_style_behavior(behavior)?;
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
        engine.sync_lwc_model_from_core()?;

        Ok(engine)
    }
}
