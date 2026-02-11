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
    LineSegment, OhlcBar, PriceScale, PriceScaleTuning, TimeScale, TimeScaleTuning, Viewport,
    candles_in_time_window, points_in_time_window, project_area_geometry, project_bars,
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
use crate::render::{Color, LinePrimitive, RenderFrame, Renderer, TextHAlign, TextPrimitive};

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

/// Style contract for the current render frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderStyle {
    pub series_line_color: Color,
    pub grid_line_color: Color,
    pub major_grid_line_color: Color,
    pub axis_border_color: Color,
    pub axis_label_color: Color,
    pub grid_line_width: f64,
    pub major_grid_line_width: f64,
    pub axis_line_width: f64,
    pub major_time_label_font_size_px: f64,
    pub price_axis_width_px: f64,
    pub time_axis_height_px: f64,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            series_line_color: Color::rgb(0.16, 0.38, 1.0),
            grid_line_color: Color::rgb(0.89, 0.92, 0.95),
            major_grid_line_color: Color::rgb(0.78, 0.83, 0.90),
            axis_border_color: Color::rgb(0.82, 0.84, 0.88),
            axis_label_color: Color::rgb(0.10, 0.12, 0.16),
            grid_line_width: 1.0,
            major_grid_line_width: 1.25,
            axis_line_width: 1.0,
            major_time_label_font_size_px: 12.0,
            price_axis_width_px: 72.0,
            time_axis_height_px: 24.0,
        }
    }
}

pub type TimeLabelFormatterFn = Arc<dyn Fn(f64) -> String + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeLabelCacheStats {
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
struct TimeLabelCacheKey {
    profile: TimeLabelCacheProfile,
    logical_time_millis: i64,
}

#[derive(Debug, Default)]
struct TimeLabelCache {
    entries: HashMap<TimeLabelCacheKey, String>,
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
    interaction: InteractionState,
    points: Vec<DataPoint>,
    candles: Vec<OhlcBar>,
    series_metadata: IndexMap<String, String>,
    plugins: Vec<Box<dyn ChartPlugin>>,
    time_axis_label_config: TimeAxisLabelConfig,
    time_label_formatter: Option<TimeLabelFormatterFn>,
    time_label_formatter_generation: u64,
    time_label_cache: RefCell<TimeLabelCache>,
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
            interaction: InteractionState::default(),
            points: Vec::new(),
            candles: Vec::new(),
            series_metadata: IndexMap::new(),
            plugins: Vec::new(),
            time_axis_label_config: TimeAxisLabelConfig::default(),
            time_label_formatter: None,
            time_label_formatter_generation: 0,
            time_label_cache: RefCell::new(TimeLabelCache::default()),
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

    #[must_use]
    pub fn time_label_cache_stats(&self) -> TimeLabelCacheStats {
        self.time_label_cache.borrow().stats()
    }

    pub fn clear_time_label_cache(&self) {
        self.time_label_cache.borrow_mut().clear();
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

    pub fn map_price_to_pixel(&self, price: f64) -> ChartResult<f64> {
        self.price_scale.price_to_pixel(price, self.viewport)
    }

    pub fn map_pixel_to_price(&self, pixel: f64) -> ChartResult<f64> {
        self.price_scale.pixel_to_price(pixel, self.viewport)
    }

    #[must_use]
    pub fn price_domain(&self) -> (f64, f64) {
        self.price_scale.domain()
    }

    pub fn autoscale_price_from_data(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_data_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from points with explicit tuning.
    pub fn autoscale_price_from_data_tuned(&mut self, tuning: PriceScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() {
            return Ok(());
        }
        self.price_scale = PriceScale::from_data_tuned(&self.points, tuning)?;
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
        self.price_scale = PriceScale::from_ohlc_tuned(&self.candles, tuning)?;
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
        let axis_color = style.axis_border_color;
        let label_color = style.axis_label_color;
        let time_tick_count =
            axis_tick_target_count(plot_right, AXIS_TIME_TARGET_SPACING_PX, 2, 12);
        let price_tick_count =
            axis_tick_target_count(plot_bottom, AXIS_PRICE_TARGET_SPACING_PX, 2, 16);

        // Axis borders remain explicit frame primitives, keeping visual output
        // deterministic across all renderer backends.
        frame = frame.with_line(LinePrimitive::new(
            0.0,
            plot_bottom,
            viewport_width,
            plot_bottom,
            style.axis_line_width,
            axis_color,
        ));
        frame = frame.with_line(LinePrimitive::new(
            plot_right,
            0.0,
            plot_right,
            viewport_height,
            style.axis_line_width,
            axis_color,
        ));

        let mut time_ticks = Vec::with_capacity(time_tick_count);
        for time in axis_ticks(self.time_scale.visible_range(), time_tick_count) {
            let px = self.time_scale.time_to_pixel(time, self.viewport)?;
            let clamped_px = px.clamp(0.0, plot_right);
            time_ticks.push((time, clamped_px));
        }

        let visible_span_abs = (visible_end - visible_start).abs();
        for (time, px) in select_ticks_with_min_spacing(time_ticks, AXIS_TIME_MIN_SPACING_PX) {
            let is_major_tick = is_major_time_tick(time, self.time_axis_label_config);
            let (grid_color, grid_line_width, label_font_size_px) = if is_major_tick {
                (
                    style.major_grid_line_color,
                    style.major_grid_line_width,
                    style.major_time_label_font_size_px,
                )
            } else {
                (style.grid_line_color, style.grid_line_width, 11.0)
            };
            let text = self.format_time_axis_label(time, visible_span_abs);
            frame = frame.with_text(TextPrimitive::new(
                text,
                px,
                (plot_bottom + 4.0).min((viewport_height - 12.0).max(0.0)),
                label_font_size_px,
                label_color,
                TextHAlign::Center,
            ));
            frame = frame.with_line(LinePrimitive::new(
                px,
                0.0,
                px,
                plot_bottom,
                grid_line_width,
                grid_color,
            ));
            frame = frame.with_line(LinePrimitive::new(
                px,
                plot_bottom,
                px,
                (plot_bottom + 6.0).min(viewport_height),
                style.axis_line_width,
                axis_color,
            ));
        }

        let mut price_ticks = Vec::with_capacity(price_tick_count);
        for price in axis_ticks(self.price_scale.domain(), price_tick_count) {
            let py = self.price_scale.price_to_pixel(price, self.viewport)?;
            let clamped_py = py.clamp(0.0, plot_bottom);
            price_ticks.push((price, clamped_py));
        }

        for (price, py) in select_ticks_with_min_spacing(price_ticks, AXIS_PRICE_MIN_SPACING_PX) {
            let text = format_axis_decimal(price, 2, self.time_axis_label_config.locale);
            frame = frame.with_text(TextPrimitive::new(
                text,
                viewport_width - 6.0,
                (py - 8.0).max(0.0),
                11.0,
                label_color,
                TextHAlign::Right,
            ));
            frame = frame.with_line(LinePrimitive::new(
                0.0,
                py,
                plot_right,
                py,
                style.grid_line_width,
                style.grid_line_color,
            ));
            frame = frame.with_line(LinePrimitive::new(
                plot_right,
                py,
                (plot_right + 6.0).min(viewport_width),
                py,
                style.axis_line_width,
                axis_color,
            ));
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
    style.major_grid_line_color.validate()?;
    style.axis_border_color.validate()?;
    style.axis_label_color.validate()?;

    for (name, value) in [
        ("grid_line_width", style.grid_line_width),
        ("major_grid_line_width", style.major_grid_line_width),
        ("axis_line_width", style.axis_line_width),
        (
            "major_time_label_font_size_px",
            style.major_time_label_font_size_px,
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
