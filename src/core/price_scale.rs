use crate::core::{DataPoint, LinearScale, OhlcBar, Viewport};
use crate::error::{ChartError, ChartResult};
use serde::{Deserialize, Serialize};

/// Mapping mode used by the price scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PriceScaleMode {
    /// Uniform spacing in raw price units.
    #[default]
    Linear,
    /// Uniform spacing in natural-log price units (all prices must be > 0).
    Log,
}

/// Tuning controls for price-domain autoscaling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PriceScaleTuning {
    pub top_padding_ratio: f64,
    pub bottom_padding_ratio: f64,
    pub min_span_absolute: f64,
}

impl Default for PriceScaleTuning {
    fn default() -> Self {
        Self {
            top_padding_ratio: 0.10,
            bottom_padding_ratio: 0.10,
            min_span_absolute: 0.000_001,
        }
    }
}

impl PriceScaleTuning {
    fn validate(self) -> ChartResult<Self> {
        if !self.top_padding_ratio.is_finite()
            || !self.bottom_padding_ratio.is_finite()
            || self.top_padding_ratio < 0.0
            || self.bottom_padding_ratio < 0.0
        {
            return Err(ChartError::InvalidData(
                "price scale padding ratios must be finite and >= 0".to_owned(),
            ));
        }

        if !self.min_span_absolute.is_finite() || self.min_span_absolute <= 0.0 {
            return Err(ChartError::InvalidData(
                "price scale min span must be finite and > 0".to_owned(),
            ));
        }

        Ok(self)
    }
}

/// Price axis model mapped to an inverted Y pixel axis.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PriceScale {
    linear: LinearScale,
    domain_start: f64,
    domain_end: f64,
    mode: PriceScaleMode,
}

impl PriceScale {
    /// Creates a price scale from explicit min/max values.
    pub fn new(price_min: f64, price_max: f64) -> ChartResult<Self> {
        Self::new_with_mode(price_min, price_max, PriceScaleMode::Linear)
    }

    /// Creates a price scale from explicit min/max values and mapping mode.
    pub fn new_with_mode(
        price_min: f64,
        price_max: f64,
        mode: PriceScaleMode,
    ) -> ChartResult<Self> {
        if !price_min.is_finite() || !price_max.is_finite() || price_min == price_max {
            return Err(ChartError::InvalidData(
                "scale domain must be finite and non-zero".to_owned(),
            ));
        }

        let transformed_start = to_scale_domain(price_min, mode)?;
        let transformed_end = to_scale_domain(price_max, mode)?;
        let linear = LinearScale::new(transformed_start, transformed_end)?;
        Ok(Self {
            linear,
            domain_start: price_min,
            domain_end: price_max,
            mode,
        })
    }

    pub fn from_data(points: &[DataPoint]) -> ChartResult<Self> {
        Self::from_data_tuned(points, PriceScaleTuning::default())
    }

    /// Computes a tuned price domain from XY points.
    pub fn from_data_tuned(points: &[DataPoint], tuning: PriceScaleTuning) -> ChartResult<Self> {
        Self::from_data_tuned_with_mode(points, tuning, PriceScaleMode::Linear)
    }

    /// Computes a tuned price domain from XY points with an explicit scale mode.
    pub fn from_data_tuned_with_mode(
        points: &[DataPoint],
        tuning: PriceScaleTuning,
        mode: PriceScaleMode,
    ) -> ChartResult<Self> {
        if points.is_empty() {
            return Err(ChartError::InvalidData(
                "price scale cannot be built from empty data".to_owned(),
            ));
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for point in points {
            if !point.y.is_finite() {
                return Err(ChartError::InvalidData(
                    "price values must be finite".to_owned(),
                ));
            }
            min = min.min(point.y);
            max = max.max(point.y);
        }

        Self::from_min_max_tuned(min, max, tuning, mode)
    }

    pub fn from_ohlc(bars: &[OhlcBar]) -> ChartResult<Self> {
        Self::from_ohlc_tuned(bars, PriceScaleTuning::default())
    }

    /// Computes a tuned price domain from OHLC bars (low/high envelope).
    pub fn from_ohlc_tuned(bars: &[OhlcBar], tuning: PriceScaleTuning) -> ChartResult<Self> {
        Self::from_ohlc_tuned_with_mode(bars, tuning, PriceScaleMode::Linear)
    }

    /// Computes a tuned price domain from OHLC bars with an explicit scale mode.
    pub fn from_ohlc_tuned_with_mode(
        bars: &[OhlcBar],
        tuning: PriceScaleTuning,
        mode: PriceScaleMode,
    ) -> ChartResult<Self> {
        if bars.is_empty() {
            return Err(ChartError::InvalidData(
                "price scale cannot be built from empty bars".to_owned(),
            ));
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for bar in bars {
            min = min.min(bar.low);
            max = max.max(bar.high);
        }

        Self::from_min_max_tuned(min, max, tuning, mode)
    }

    #[must_use]
    /// Returns the raw price domain kept by the scale.
    pub fn domain(self) -> (f64, f64) {
        (self.domain_start, self.domain_end)
    }

    #[must_use]
    /// Returns the active mapping mode.
    pub fn mode(self) -> PriceScaleMode {
        self.mode
    }

    /// Rebuilds this scale using the same raw domain and a different mapping mode.
    pub fn with_mode(self, mode: PriceScaleMode) -> ChartResult<Self> {
        Self::new_with_mode(self.domain_start, self.domain_end, mode)
    }

    /// Builds axis ticks in the active transformed domain, then maps back to raw prices.
    pub fn ticks(self, tick_count: usize) -> ChartResult<Vec<f64>> {
        if tick_count == 0 {
            return Ok(Vec::new());
        }
        if tick_count == 1 {
            return Ok(vec![self.domain_start]);
        }

        let mut ticks = Vec::with_capacity(tick_count);
        let transformed = self.linear.domain();
        let span = transformed.1 - transformed.0;
        let denominator = (tick_count - 1) as f64;
        for index in 0..tick_count {
            let ratio = (index as f64) / denominator;
            let transformed_value = transformed.0 + span * ratio;
            ticks.push(from_scale_domain(transformed_value, self.mode)?);
        }
        Ok(ticks)
    }

    /// Maps a raw price to pixel Y, preserving inverted-axis behavior.
    pub fn price_to_pixel(self, price: f64, viewport: Viewport) -> ChartResult<f64> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }

        // Price is mapped on the vertical axis, so we reuse LinearScale with
        // `height` as the logical width and then invert the direction.
        let axis_viewport = Viewport::new(viewport.height, 1);
        let transformed_price = to_scale_domain(price, self.mode)?;
        let y_from_bottom = self
            .linear
            .domain_to_pixel(transformed_price, axis_viewport)?;
        Ok(f64::from(viewport.height) - y_from_bottom)
    }

    pub fn pixel_to_price(self, pixel: f64, viewport: Viewport) -> ChartResult<f64> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }

        if !pixel.is_finite() {
            return Err(ChartError::InvalidData("pixel must be finite".to_owned()));
        }

        let axis_viewport = Viewport::new(viewport.height, 1);
        let y_from_bottom = f64::from(viewport.height) - pixel;
        let transformed_price = self.linear.pixel_to_domain(y_from_bottom, axis_viewport)?;
        from_scale_domain(transformed_price, self.mode)
    }

    fn from_min_max_tuned(
        min: f64,
        max: f64,
        tuning: PriceScaleTuning,
        mode: PriceScaleMode,
    ) -> ChartResult<Self> {
        let tuning = tuning.validate()?;
        match mode {
            PriceScaleMode::Linear => {
                let (base_min, base_max) = normalize_range(min, max, tuning.min_span_absolute)?;
                let span = base_max - base_min;

                let padded_min = base_min - span * tuning.bottom_padding_ratio;
                let padded_max = base_max + span * tuning.top_padding_ratio;
                let normalized = normalize_range(padded_min, padded_max, tuning.min_span_absolute)?;

                Self::new_with_mode(normalized.0, normalized.1, mode)
            }
            PriceScaleMode::Log => {
                let log_min = to_scale_domain(min, mode)?;
                let log_max = to_scale_domain(max, mode)?;
                // Preserve the "minimum span" intent by approximating the additive
                // raw-price span as a multiplicative span in log space.
                let min_log_span = {
                    let candidate = (min + tuning.min_span_absolute).ln() - min.ln();
                    if candidate.is_finite() && candidate > 0.0 {
                        candidate
                    } else {
                        f64::EPSILON
                    }
                };
                let (base_min, base_max) = normalize_range(log_min, log_max, min_log_span)?;
                let span = base_max - base_min;
                let padded_min = base_min - span * tuning.bottom_padding_ratio;
                let padded_max = base_max + span * tuning.top_padding_ratio;
                let normalized = normalize_range(padded_min, padded_max, min_log_span)?;

                let domain_min = from_scale_domain(normalized.0, mode)?;
                let domain_max = from_scale_domain(normalized.1, mode)?;
                Self::new_with_mode(domain_min, domain_max, mode)
            }
        }
    }
}

/// Maps raw price values into the internal scale domain selected by `mode`.
fn to_scale_domain(value: f64, mode: PriceScaleMode) -> ChartResult<f64> {
    if !value.is_finite() {
        return Err(ChartError::InvalidData("price must be finite".to_owned()));
    }

    match mode {
        PriceScaleMode::Linear => Ok(value),
        PriceScaleMode::Log => {
            if value <= 0.0 {
                return Err(ChartError::InvalidData(
                    "log price scale requires values > 0".to_owned(),
                ));
            }
            Ok(value.ln())
        }
    }
}

/// Maps internal scale-domain values back into raw price values.
fn from_scale_domain(value: f64, mode: PriceScaleMode) -> ChartResult<f64> {
    if !value.is_finite() {
        return Err(ChartError::InvalidData(
            "mapped scale value must be finite".to_owned(),
        ));
    }

    match mode {
        PriceScaleMode::Linear => Ok(value),
        PriceScaleMode::Log => {
            let raw = value.exp();
            if !raw.is_finite() || raw <= 0.0 {
                return Err(ChartError::InvalidData(
                    "mapped log price must be finite and > 0".to_owned(),
                ));
            }
            Ok(raw)
        }
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
