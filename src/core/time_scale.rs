use crate::core::{DataPoint, LinearScale, OhlcBar, Viewport};
use crate::error::{ChartError, ChartResult};
use serde::{Deserialize, Serialize};

/// Lightweight-style logical-index coordinate space.
///
/// This helper mirrors the `TimeScale` index/coordinate formulas used by
/// Lightweight Charts internals (`indexToCoordinate` / `coordinateToIndex`),
/// including center-of-bar (`+0.5`) and right-edge pixel (`-1`) offsets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeIndexCoordinateSpace {
    pub base_index: f64,
    pub right_offset_bars: f64,
    pub bar_spacing_px: f64,
    pub width_px: f64,
}

impl TimeIndexCoordinateSpace {
    /// Maps logical bar index to canvas X coordinate.
    ///
    /// Formula:
    /// `x = width - (base + rightOffset - index + 0.5) * barSpacing - 1`
    pub fn index_to_coordinate(self, logical_index: f64) -> ChartResult<f64> {
        self.validate()?;
        if !logical_index.is_finite() {
            return Err(ChartError::InvalidData(
                "time logical index must be finite".to_owned(),
            ));
        }
        Ok(self.width_px
            - (self.base_index + self.right_offset_bars - logical_index + 0.5)
                * self.bar_spacing_px
            - 1.0)
    }

    /// Maps canvas X coordinate to floating logical bar index.
    ///
    /// Inverse formula:
    /// `index = base + rightOffset + 0.5 - (width - x - 1) / barSpacing`
    pub fn coordinate_to_logical_index(self, coordinate_px: f64) -> ChartResult<f64> {
        self.validate()?;
        if !coordinate_px.is_finite() {
            return Err(ChartError::InvalidData(
                "time coordinate must be finite".to_owned(),
            ));
        }
        Ok(self.base_index + self.right_offset_bars + 0.5
            - (self.width_px - coordinate_px - 1.0) / self.bar_spacing_px)
    }

    /// Maps canvas X coordinate to discrete logical index using ceil semantics.
    pub fn coordinate_to_index_ceil(self, coordinate_px: f64) -> ChartResult<i64> {
        let logical = self.coordinate_to_logical_index(coordinate_px)?;
        if logical < (i64::MIN as f64) || logical > (i64::MAX as f64) {
            return Err(ChartError::InvalidData(
                "time logical index exceeds i64 range".to_owned(),
            ));
        }
        Ok(logical.ceil() as i64)
    }

    /// Resolves nearest filled slot for sparse logical-index datasets.
    ///
    /// This mirrors Lightweight's "ignore whitespace indices" behavior by
    /// selecting the nearest available data slot instead of returning a hole.
    ///
    /// The caller provides monotonic ascending logical indices via `index_at`.
    pub fn coordinate_to_nearest_filled_slot<F>(
        self,
        coordinate_px: f64,
        len: usize,
        mut index_at: F,
    ) -> ChartResult<Option<usize>>
    where
        F: FnMut(usize) -> f64,
    {
        self.validate()?;
        if !coordinate_px.is_finite() {
            return Err(ChartError::InvalidData(
                "time coordinate must be finite".to_owned(),
            ));
        }
        if len == 0 {
            return Ok(None);
        }

        let logical = self.coordinate_to_logical_index(coordinate_px)?;
        let mut left = 0usize;
        let mut right = len;
        while left < right {
            let mid = left + (right - left) / 2;
            let mid_value = index_at(mid);
            if !mid_value.is_finite() {
                return Err(ChartError::InvalidData(
                    "filled logical indices must be finite".to_owned(),
                ));
            }
            if mid_value < logical {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        if left == 0 {
            return Ok(Some(0));
        }
        if left >= len {
            return Ok(Some(len - 1));
        }

        let lower_slot = left - 1;
        let upper_slot = left;
        let lower_index = index_at(lower_slot);
        let upper_index = index_at(upper_slot);
        if !lower_index.is_finite() || !upper_index.is_finite() {
            return Err(ChartError::InvalidData(
                "filled logical indices must be finite".to_owned(),
            ));
        }

        let lower_distance = (logical - lower_index).abs();
        let upper_distance = (upper_index - logical).abs();
        if upper_distance <= lower_distance {
            Ok(Some(upper_slot))
        } else {
            Ok(Some(lower_slot))
        }
    }

    /// Applies pixel pan delta to right offset in bars.
    ///
    /// Formula: `rightOffset += deltaPx / barSpacing`
    pub fn pan_right_offset_by_pixels(self, delta_px: f64) -> ChartResult<f64> {
        self.validate()?;
        if !delta_px.is_finite() {
            return Err(ChartError::InvalidData(
                "pan delta px must be finite".to_owned(),
            ));
        }
        Ok(self.right_offset_bars + delta_px / self.bar_spacing_px)
    }

    /// Solves new right offset for anchor-preserving zoom.
    ///
    /// Given `old_bar_spacing_px`, this computes the right offset that keeps
    /// `anchor_logical_index` at the same screen X after switching to
    /// `self.bar_spacing_px`.
    pub fn solve_right_offset_for_anchor_preserving_zoom(
        self,
        old_bar_spacing_px: f64,
        old_right_offset_bars: f64,
        anchor_logical_index: f64,
    ) -> ChartResult<f64> {
        self.validate()?;
        if !old_bar_spacing_px.is_finite() || old_bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "old bar spacing must be finite and > 0".to_owned(),
            ));
        }
        if !old_right_offset_bars.is_finite() {
            return Err(ChartError::InvalidData(
                "old right offset must be finite".to_owned(),
            ));
        }
        if !anchor_logical_index.is_finite() {
            return Err(ChartError::InvalidData(
                "zoom anchor logical index must be finite".to_owned(),
            ));
        }

        let old_distance_bars =
            self.base_index + old_right_offset_bars - anchor_logical_index + 0.5;
        Ok(
            old_distance_bars * (old_bar_spacing_px / self.bar_spacing_px) - self.base_index
                + anchor_logical_index
                - 0.5,
        )
    }

    fn validate(self) -> ChartResult<()> {
        if !self.base_index.is_finite() {
            return Err(ChartError::InvalidData(
                "base index must be finite".to_owned(),
            ));
        }
        if !self.right_offset_bars.is_finite() {
            return Err(ChartError::InvalidData(
                "right offset bars must be finite".to_owned(),
            ));
        }
        if !self.bar_spacing_px.is_finite() || self.bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "bar spacing px must be finite and > 0".to_owned(),
            ));
        }
        if !self.width_px.is_finite() || self.width_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "width px must be finite and > 0".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Tuning controls for visible time range fitting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeScaleTuning {
    pub left_padding_ratio: f64,
    pub right_padding_ratio: f64,
    pub min_span_absolute: f64,
}

impl Default for TimeScaleTuning {
    fn default() -> Self {
        Self {
            left_padding_ratio: 0.05,
            right_padding_ratio: 0.05,
            min_span_absolute: 1.0,
        }
    }
}

impl TimeScaleTuning {
    fn validate(self) -> ChartResult<Self> {
        if !self.left_padding_ratio.is_finite()
            || !self.right_padding_ratio.is_finite()
            || self.left_padding_ratio < 0.0
            || self.right_padding_ratio < 0.0
        {
            return Err(ChartError::InvalidData(
                "time scale padding ratios must be finite and >= 0".to_owned(),
            ));
        }

        if !self.min_span_absolute.is_finite() || self.min_span_absolute <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale min span must be finite and > 0".to_owned(),
            ));
        }

        Ok(self)
    }
}

/// Time axis model with separate full and visible ranges.
///
/// `full_*` tracks the raw fitted data range.
/// `visible_*` includes optional padding and user-driven range changes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeScale {
    full_start: f64,
    full_end: f64,
    visible_start: f64,
    visible_end: f64,
}

impl TimeScale {
    /// Creates a scale with matching full and visible ranges.
    pub fn new(time_start: f64, time_end: f64) -> ChartResult<Self> {
        let normalized = normalize_range(time_start, time_end, 1.0)?;
        Ok(Self {
            full_start: normalized.0,
            full_end: normalized.1,
            visible_start: normalized.0,
            visible_end: normalized.1,
        })
    }

    pub fn from_data(points: &[DataPoint]) -> ChartResult<Self> {
        Self::from_data_tuned(points, TimeScaleTuning::default())
    }

    /// Fits full/visible ranges from XY data points using explicit tuning.
    pub fn from_data_tuned(points: &[DataPoint], tuning: TimeScaleTuning) -> ChartResult<Self> {
        Self::from_mixed_data_tuned(points, &[], tuning)
    }

    /// Fits full/visible ranges from a mixed data source (points + candles).
    pub fn from_mixed_data_tuned(
        points: &[DataPoint],
        bars: &[OhlcBar],
        tuning: TimeScaleTuning,
    ) -> ChartResult<Self> {
        let tuning = tuning.validate()?;

        if points.is_empty() && bars.is_empty() {
            return Err(ChartError::InvalidData(
                "time scale cannot be built from empty data".to_owned(),
            ));
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for point in points {
            if !point.x.is_finite() {
                return Err(ChartError::InvalidData(
                    "time values must be finite".to_owned(),
                ));
            }
            min = min.min(point.x);
            max = max.max(point.x);
        }

        for bar in bars {
            if !bar.time.is_finite() {
                return Err(ChartError::InvalidData(
                    "candle times must be finite".to_owned(),
                ));
            }
            min = min.min(bar.time);
            max = max.max(bar.time);
        }

        let (full_start, full_end) = normalize_range(min, max, tuning.min_span_absolute)?;
        let full_span = full_end - full_start;
        let visible_start = full_start - full_span * tuning.left_padding_ratio;
        let visible_end = full_end + full_span * tuning.right_padding_ratio;

        Ok(Self {
            full_start,
            full_end,
            visible_start,
            visible_end,
        })
    }

    #[must_use]
    pub fn domain(self) -> (f64, f64) {
        (self.visible_start, self.visible_end)
    }

    #[must_use]
    pub fn full_range(self) -> (f64, f64) {
        (self.full_start, self.full_end)
    }

    #[must_use]
    pub fn visible_range(self) -> (f64, f64) {
        (self.visible_start, self.visible_end)
    }

    /// Overrides the visible range without modifying the full fitted range.
    pub fn set_visible_range(&mut self, start: f64, end: f64) -> ChartResult<()> {
        let normalized = normalize_range(start, end, 1e-9)?;
        self.visible_start = normalized.0;
        self.visible_end = normalized.1;
        Ok(())
    }

    /// Applies optional visible-range constraints against full-range edges.
    ///
    /// - `fix_left_edge`: keeps visible start at or after full start
    /// - `fix_right_edge`: keeps visible end at or before full end
    ///
    /// When both flags are enabled and visible span is wider than full span,
    /// visible range is clamped to full range.
    pub fn clamp_visible_range_to_full_edges(
        &mut self,
        fix_left_edge: bool,
        fix_right_edge: bool,
    ) -> ChartResult<bool> {
        if !fix_left_edge && !fix_right_edge {
            return Ok(false);
        }

        let full_start = self.full_start;
        let full_end = self.full_end;
        let full_span = full_end - full_start;

        let mut start = self.visible_start;
        let mut end = self.visible_end;
        let span = end - start;

        if fix_left_edge && fix_right_edge && span >= full_span {
            start = full_start;
            end = full_end;
        } else if fix_left_edge && fix_right_edge {
            if start < full_start {
                start = full_start;
                end = start + span;
            }
            if end > full_end {
                end = full_end;
                start = end - span;
            }
        } else if fix_left_edge {
            if start < full_start {
                let shift = full_start - start;
                start += shift;
                end += shift;
            }
        } else if fix_right_edge && end > full_end {
            let shift = end - full_end;
            start -= shift;
            end -= shift;
        }

        let changed =
            (start - self.visible_start).abs() > 1e-12 || (end - self.visible_end).abs() > 1e-12;
        if changed {
            self.set_visible_range(start, end)?;
        }
        Ok(changed)
    }

    pub fn reset_visible_range_to_full(&mut self) {
        self.visible_start = self.full_start;
        self.visible_end = self.full_end;
    }

    /// Extends the fitted full range to include a new time sample.
    ///
    /// Visible range is intentionally not modified here.
    pub fn include_time_in_full_range(
        &mut self,
        time: f64,
        min_span_absolute: f64,
    ) -> ChartResult<bool> {
        if !time.is_finite() {
            return Err(ChartError::InvalidData(
                "time value must be finite".to_owned(),
            ));
        }
        if !min_span_absolute.is_finite() || min_span_absolute <= 0.0 {
            return Err(ChartError::InvalidData(
                "time min span must be finite and > 0".to_owned(),
            ));
        }

        let previous_start = self.full_start;
        let previous_end = self.full_end;

        let start = self.full_start.min(time);
        let end = self.full_end.max(time);
        let normalized = normalize_range(start, end, min_span_absolute)?;
        self.full_start = normalized.0;
        self.full_end = normalized.1;

        Ok((self.full_start - previous_start).abs() > 1e-12
            || (self.full_end - previous_end).abs() > 1e-12)
    }

    /// Pans the visible range by an additive time delta.
    pub fn pan_visible_by_delta(&mut self, delta_time: f64) -> ChartResult<()> {
        if !delta_time.is_finite() {
            return Err(ChartError::InvalidData(
                "pan delta must be finite".to_owned(),
            ));
        }

        self.visible_start += delta_time;
        self.visible_end += delta_time;
        Ok(())
    }

    /// Zooms visible range around an anchor time.
    ///
    /// `factor > 1.0` zooms in, `0.0 < factor < 1.0` zooms out.
    /// The resulting span is clamped by `min_span_absolute`.
    pub fn zoom_visible_by_factor(
        &mut self,
        factor: f64,
        anchor_time: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(ChartError::InvalidData(
                "zoom factor must be finite and > 0".to_owned(),
            ));
        }
        if !anchor_time.is_finite() {
            return Err(ChartError::InvalidData(
                "zoom anchor must be finite".to_owned(),
            ));
        }
        if !min_span_absolute.is_finite() || min_span_absolute <= 0.0 {
            return Err(ChartError::InvalidData(
                "zoom min span must be finite and > 0".to_owned(),
            ));
        }

        let current_span = self.visible_end - self.visible_start;
        let target_span = (current_span / factor).max(min_span_absolute);
        let left_ratio = (anchor_time - self.visible_start) / current_span;

        let new_start = anchor_time - left_ratio * target_span;
        let new_end = new_start + target_span;
        self.set_visible_range(new_start, new_end)
    }

    /// Re-fits the scale from mixed data and applies tuning.
    pub fn fit_to_mixed_data(
        &mut self,
        points: &[DataPoint],
        bars: &[OhlcBar],
        tuning: TimeScaleTuning,
    ) -> ChartResult<()> {
        let fitted = Self::from_mixed_data_tuned(points, bars, tuning)?;
        *self = fitted;
        Ok(())
    }

    pub fn fit_to_data(
        &mut self,
        points: &[DataPoint],
        tuning: TimeScaleTuning,
    ) -> ChartResult<()> {
        let fitted = Self::from_data_tuned(points, tuning)?;
        *self = fitted;
        Ok(())
    }

    /// Derives current visible bar spacing and right offset in bar units.
    ///
    /// This mirrors the internal time-scale state model used by Lightweight:
    /// visible range can be represented as `{barSpacing, rightOffset}` against
    /// full-range right edge and a reference bar step.
    pub fn derive_visible_bar_spacing_and_right_offset(
        self,
        reference_step: f64,
        viewport_width_px: f64,
    ) -> ChartResult<(f64, f64)> {
        if !reference_step.is_finite() || reference_step <= 0.0 {
            return Err(ChartError::InvalidData(
                "reference step must be finite and > 0".to_owned(),
            ));
        }
        if !viewport_width_px.is_finite() || viewport_width_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "viewport width must be finite and > 0".to_owned(),
            ));
        }

        let visible_span = self.visible_end - self.visible_start;
        if !visible_span.is_finite() || visible_span <= 0.0 {
            return Err(ChartError::InvalidData(
                "visible time span must be finite and > 0".to_owned(),
            ));
        }

        let visible_bars = visible_span / reference_step;
        if !visible_bars.is_finite() || visible_bars <= 0.0 {
            return Err(ChartError::InvalidData(
                "visible bars must be finite and > 0".to_owned(),
            ));
        }

        let bar_spacing_px = viewport_width_px / visible_bars;
        if !bar_spacing_px.is_finite() || bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "derived bar spacing must be finite and > 0".to_owned(),
            ));
        }

        let right_offset_bars = (self.visible_end - self.full_end) / reference_step;
        if !right_offset_bars.is_finite() {
            return Err(ChartError::InvalidData(
                "derived right offset must be finite".to_owned(),
            ));
        }

        Ok((bar_spacing_px, right_offset_bars))
    }

    /// Rebuilds visible range from explicit bar spacing and right offset.
    pub fn set_visible_range_from_bar_spacing_and_right_offset(
        &mut self,
        bar_spacing_px: f64,
        right_offset_bars: f64,
        reference_step: f64,
        viewport_width_px: f64,
    ) -> ChartResult<()> {
        if !bar_spacing_px.is_finite() || bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "bar spacing must be finite and > 0".to_owned(),
            ));
        }
        if !right_offset_bars.is_finite() {
            return Err(ChartError::InvalidData(
                "right offset bars must be finite".to_owned(),
            ));
        }
        if !reference_step.is_finite() || reference_step <= 0.0 {
            return Err(ChartError::InvalidData(
                "reference step must be finite and > 0".to_owned(),
            ));
        }
        if !viewport_width_px.is_finite() || viewport_width_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "viewport width must be finite and > 0".to_owned(),
            ));
        }

        let visible_bars = (viewport_width_px / bar_spacing_px).max(f64::EPSILON);
        let visible_span = reference_step * visible_bars;
        let target_end = self.full_end + right_offset_bars * reference_step;
        let target_start = target_end - visible_span;
        self.set_visible_range(target_start, target_end)
    }

    pub fn time_to_pixel(self, time: f64, viewport: Viewport) -> ChartResult<f64> {
        self.visible_linear()?.domain_to_pixel(time, viewport)
    }

    pub fn pixel_to_time(self, pixel: f64, viewport: Viewport) -> ChartResult<f64> {
        self.visible_linear()?.pixel_to_domain(pixel, viewport)
    }

    fn visible_linear(self) -> ChartResult<LinearScale> {
        LinearScale::new(self.visible_start, self.visible_end)
    }
}

fn normalize_range(start: f64, end: f64, min_span: f64) -> ChartResult<(f64, f64)> {
    if !start.is_finite() || !end.is_finite() {
        return Err(ChartError::InvalidData(
            "scale range must be finite".to_owned(),
        ));
    }

    if start == end {
        let half = min_span / 2.0;
        return Ok((start - half, end + half));
    }

    Ok((start.min(end), start.max(end)))
}
