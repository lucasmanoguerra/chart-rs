use crate::core::{DataPoint, LinearScale, OhlcBar, Viewport};
use crate::error::{ChartError, ChartResult};

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScale {
    full_start: f64,
    full_end: f64,
    visible_start: f64,
    visible_end: f64,
}

impl TimeScale {
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

    pub fn from_data_tuned(points: &[DataPoint], tuning: TimeScaleTuning) -> ChartResult<Self> {
        Self::from_mixed_data_tuned(points, &[], tuning)
    }

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
