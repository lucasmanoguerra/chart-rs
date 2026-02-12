use std::cell::RefCell;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::core::{
    CandleGeometry, DataPoint, OhlcBar, PriceScale, PriceScaleMode, TimeScale, Viewport,
};
use crate::error::{ChartError, ChartResult};
use crate::extensions::{ChartPlugin, PluginEvent};
use crate::interaction::{CrosshairState, InteractionState};
use crate::render::Renderer;

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
mod data_controller;
mod engine_accessors;
mod interaction_controller;
mod label_formatter_controller;
mod plugin_dispatch;
mod plugin_registry;
mod price_resolver;
mod price_scale_access;
mod render_frame_builder;
mod scale_access;
mod series_projection;
mod snap_resolver;
mod snapshot_controller;
mod time_scale_controller;
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
        })
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
