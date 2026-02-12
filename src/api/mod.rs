use std::cell::RefCell;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
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
    CrosshairMode, CrosshairState, InteractionMode, InteractionState, KineticPanConfig,
    KineticPanState,
};
use crate::render::{
    Color, LinePrimitive, RectPrimitive, RenderFrame, Renderer, TextHAlign, TextPrimitive,
};

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
use label_cache::{
    PriceLabelCache, PriceLabelCacheKey, PriceLabelCacheProfile, TimeLabelCache, TimeLabelCacheKey,
    TimeLabelCacheProfile, TimeLabelPattern, price_policy_profile,
};
pub use label_cache::{
    PriceLabelCacheStats, PriceLabelFormatterFn, TimeLabelCacheStats, TimeLabelFormatterFn,
};

mod validation;
use validation::{
    validate_price_axis_label_config, validate_render_style, validate_time_axis_label_config,
};

mod axis_label_format;
use axis_label_format::{
    ResolvedTimeLabelPattern, format_price_axis_label, format_time_axis_label, is_major_time_tick,
    map_price_step_to_display_value, map_price_to_display_value, price_display_mode_suffix,
    quantize_logical_time_millis, quantize_price_label_value, resolve_time_label_pattern,
};

mod axis_ticks;
use axis_ticks::{
    AXIS_PRICE_MIN_SPACING_PX, AXIS_PRICE_TARGET_SPACING_PX, AXIS_TIME_MIN_SPACING_PX,
    AXIS_TIME_TARGET_SPACING_PX, axis_tick_target_count, axis_ticks, select_ticks_with_min_spacing,
    tick_step_hint_from_values,
};

mod data_window;
use data_window::{expand_visible_window, markers_in_time_window};

mod interaction_validation;
use interaction_validation::validate_kinetic_pan_config;

mod layout_helpers;
use layout_helpers::{
    estimate_label_text_width_px, rects_overlap, resolve_crosshair_box_vertical_layout,
    stabilize_position,
};

mod price_resolver;
mod snap_resolver;

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
                    let estimated_text_width =
                        estimate_label_text_width_px(&text, style.last_price_label_font_size_px);
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
                frame = frame.with_line(
                    LinePrimitive::new(
                        crosshair_x,
                        0.0,
                        crosshair_x,
                        plot_bottom,
                        style.crosshair_line_width,
                        style.crosshair_line_color,
                    )
                    .with_stroke_style(
                        style
                            .crosshair_vertical_line_style
                            .unwrap_or(style.crosshair_line_style),
                    ),
                );
            }
            if style.show_crosshair_horizontal_line {
                frame = frame.with_line(
                    LinePrimitive::new(
                        0.0,
                        crosshair_y,
                        plot_right,
                        crosshair_y,
                        style.crosshair_line_width,
                        style.crosshair_line_color,
                    )
                    .with_stroke_style(
                        style
                            .crosshair_horizontal_line_style
                            .unwrap_or(style.crosshair_line_style),
                    ),
                );
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
                    stabilize_position(crosshair_time_label_x, time_stabilization_step).clamp(
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
                    let estimated_text_width = estimate_label_text_width_px(
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
                        resolve_crosshair_box_vertical_layout(
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
                    stabilize_position(price_label_anchor_y, price_stabilization_step).max(0.0);
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
                    let estimated_text_width = estimate_label_text_width_px(
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
                        resolve_crosshair_box_vertical_layout(
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
                if rects_overlap(time_rect, price_rect) {
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
            let mut z_order_policy = style.crosshair_label_box_z_order_policy;
            if let Some(time_policy) = style.crosshair_time_label_box_z_order_policy {
                z_order_policy = time_policy;
            }
            if let Some(price_policy) = style.crosshair_price_label_box_z_order_policy {
                z_order_policy = price_policy;
            }
            match z_order_policy {
                CrosshairLabelBoxZOrderPolicy::PriceAboveTime => {
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
                CrosshairLabelBoxZOrderPolicy::TimeAbovePrice => {
                    if let Some(rect) = price_box_rect {
                        frame = frame.with_rect(rect);
                    }
                    if let Some(rect) = time_box_rect {
                        frame = frame.with_rect(rect);
                    }
                    if let Some(text) = price_box_text {
                        frame = frame.with_text(text);
                    }
                    if let Some(text) = time_box_text {
                        frame = frame.with_text(text);
                    }
                }
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
}
