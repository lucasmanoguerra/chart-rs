use crate::core::{DataPoint, LinearScale, OhlcBar, Viewport};
use crate::error::{ChartError, ChartResult};
use serde::{Deserialize, Serialize};

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

    pub fn reset_visible_range_to_full(&mut self) {
        self.visible_start = self.full_start;
        self.visible_end = self.full_end;
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
