use crate::core::{
    AreaGeometry, BarGeometry, BaselineGeometry, CandleGeometry, HistogramBar, LineSegment,
    candles_in_time_window, points_in_time_window, project_area_geometry, project_bars,
    project_baseline_geometry, project_candles, project_histogram_bars, project_line_segments,
};
use crate::error::ChartResult;
use crate::extensions::{
    MarkerPlacementConfig, PlacedMarker, SeriesMarker, place_markers_on_candles,
};
use crate::render::Renderer;

use super::ChartEngine;
use super::data_window::{expand_visible_window, markers_in_time_window};

impl<R: Renderer> ChartEngine<R> {
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
}
