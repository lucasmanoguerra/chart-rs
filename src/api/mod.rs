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
use crate::interaction::{CrosshairSnap, CrosshairState, InteractionMode, InteractionState};
use crate::render::{RenderFrame, Renderer};

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
    interaction: InteractionState,
    points: Vec<DataPoint>,
    candles: Vec<OhlcBar>,
    series_metadata: IndexMap<String, String>,
    plugins: Vec<Box<dyn ChartPlugin>>,
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

    #[must_use]
    pub fn interaction_mode(&self) -> InteractionMode {
        self.interaction.mode()
    }

    #[must_use]
    pub fn crosshair_state(&self) -> CrosshairState {
        self.interaction.crosshair()
    }

    /// Handles pointer movement and updates crosshair snapping in one step.
    pub fn pointer_move(&mut self, x: f64, y: f64) {
        self.interaction.on_pointer_move(x, y);
        self.interaction.set_crosshair_snap(self.snap_at_x(x));
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

    pub fn render(&mut self) -> ChartResult<()> {
        let frame = RenderFrame::new(self.viewport, self.points.clone());
        self.renderer.render(&frame)?;
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
