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
    #[serde(default)]
    inverted: bool,
    #[serde(default)]
    top_margin_ratio: f64,
    #[serde(default)]
    bottom_margin_ratio: f64,
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
            inverted: false,
            top_margin_ratio: 0.0,
            bottom_margin_ratio: 0.0,
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

    #[must_use]
    /// Returns whether the pixel mapping direction is inverted.
    pub fn is_inverted(self) -> bool {
        self.inverted
    }

    #[must_use]
    /// Returns a copy with updated inverted-axis behavior.
    pub fn with_inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    #[must_use]
    pub fn margins(self) -> (f64, f64) {
        (self.top_margin_ratio, self.bottom_margin_ratio)
    }

    pub fn with_margins(
        mut self,
        top_margin_ratio: f64,
        bottom_margin_ratio: f64,
    ) -> ChartResult<Self> {
        validate_scale_margins(top_margin_ratio, bottom_margin_ratio)?;
        self.top_margin_ratio = top_margin_ratio;
        self.bottom_margin_ratio = bottom_margin_ratio;
        Ok(self)
    }

    /// Rebuilds this scale using the same raw domain and a different mapping mode.
    pub fn with_mode(self, mode: PriceScaleMode) -> ChartResult<Self> {
        let mut rebuilt = Self::new_with_mode(self.domain_start, self.domain_end, mode)?;
        rebuilt.inverted = self.inverted;
        rebuilt.top_margin_ratio = self.top_margin_ratio;
        rebuilt.bottom_margin_ratio = self.bottom_margin_ratio;
        Ok(rebuilt)
    }

    /// Builds axis ticks in the active transformed domain, then maps back to raw prices.
    pub fn ticks(self, tick_count: usize) -> ChartResult<Vec<f64>> {
        if tick_count == 0 {
            return Ok(Vec::new());
        }
        if tick_count == 1 {
            return Ok(vec![self.domain_start]);
        }

        match self.mode {
            PriceScaleMode::Linear => {
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
            PriceScaleMode::Log => {
                let mut ticks = log_ladder_ticks(self.domain_start, self.domain_end, tick_count)?;
                if ticks.len() > tick_count {
                    ticks = evenly_sample_ticks(ticks, tick_count);
                }
                Ok(ticks)
            }
        }
    }

    /// Maps a raw price to pixel Y, preserving inverted-axis behavior.
    pub fn price_to_pixel(self, price: f64, viewport: Viewport) -> ChartResult<f64> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }

        let transformed_price = to_scale_domain(price, self.mode)?;
        let (top_px, bottom_px, plot_height) = resolve_price_axis_margins_px(
            viewport,
            self.top_margin_ratio,
            self.bottom_margin_ratio,
        )?;
        let (domain_start, domain_end) = self.linear.domain();
        let normalized = (transformed_price - domain_start) / (domain_end - domain_start);
        let y_from_bottom = normalized * plot_height;
        if self.inverted {
            Ok(top_px + y_from_bottom)
        } else {
            Ok(f64::from(viewport.height) - bottom_px - y_from_bottom)
        }
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

        let (top_px, bottom_px, plot_height) = resolve_price_axis_margins_px(
            viewport,
            self.top_margin_ratio,
            self.bottom_margin_ratio,
        )?;
        let y_from_bottom = if self.inverted {
            pixel - top_px
        } else {
            (f64::from(viewport.height) - bottom_px) - pixel
        };
        let (domain_start, domain_end) = self.linear.domain();
        let normalized = y_from_bottom / plot_height;
        let transformed_price = domain_start + normalized * (domain_end - domain_start);
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

fn validate_scale_margins(top_margin_ratio: f64, bottom_margin_ratio: f64) -> ChartResult<()> {
    if !top_margin_ratio.is_finite()
        || !bottom_margin_ratio.is_finite()
        || top_margin_ratio < 0.0
        || bottom_margin_ratio < 0.0
    {
        return Err(ChartError::InvalidData(
            "price scale margins must be finite and >= 0".to_owned(),
        ));
    }
    if top_margin_ratio + bottom_margin_ratio >= 1.0 {
        return Err(ChartError::InvalidData(
            "price scale margins must sum to < 1".to_owned(),
        ));
    }
    Ok(())
}

fn resolve_price_axis_margins_px(
    viewport: Viewport,
    top_margin_ratio: f64,
    bottom_margin_ratio: f64,
) -> ChartResult<(f64, f64, f64)> {
    validate_scale_margins(top_margin_ratio, bottom_margin_ratio)?;
    let height = f64::from(viewport.height);
    let top_px = height * top_margin_ratio;
    let bottom_px = height * bottom_margin_ratio;
    let plot_height = height - top_px - bottom_px;
    if !plot_height.is_finite() || plot_height <= 0.0 {
        return Err(ChartError::InvalidData(
            "price scale effective plot height must be finite and > 0".to_owned(),
        ));
    }
    Ok((top_px, bottom_px, plot_height))
}

fn log_ladder_ticks(start: f64, end: f64, tick_count: usize) -> ChartResult<Vec<f64>> {
    if start <= 0.0 || end <= 0.0 {
        return Err(ChartError::InvalidData(
            "log price scale requires values > 0".to_owned(),
        ));
    }

    let ascending = start <= end;
    let min = start.min(end);
    let max = start.max(end);
    let min_exp = min.log10().floor() as i32;
    let max_exp = max.log10().ceil() as i32;

    let mut ticks = Vec::new();
    for exp in min_exp..=max_exp {
        let decade = 10_f64.powi(exp);
        for multiplier in [1.0, 2.0, 5.0] {
            let candidate = decade * multiplier;
            if candidate >= min && candidate <= max {
                ticks.push(candidate);
            }
        }
    }

    if !ticks.iter().any(|value| approx_equal(*value, min)) {
        ticks.push(min);
    }
    if !ticks.iter().any(|value| approx_equal(*value, max)) {
        ticks.push(max);
    }

    ticks.sort_by(|lhs, rhs| lhs.total_cmp(rhs));
    ticks.dedup_by(|lhs, rhs| approx_equal(*lhs, *rhs));

    let mut sampled = if ticks.len() > tick_count {
        evenly_sample_ticks(ticks, tick_count)
    } else {
        ticks
    };
    if !ascending {
        sampled.reverse();
    }
    Ok(sampled)
}

fn evenly_sample_ticks(ticks: Vec<f64>, target: usize) -> Vec<f64> {
    if ticks.len() <= target || target == 0 {
        return ticks;
    }
    if target == 1 {
        return vec![ticks[0]];
    }

    let last_index = ticks.len() - 1;
    let mut sampled = Vec::with_capacity(target);
    for step in 0..target {
        let ratio = (step as f64) / ((target - 1) as f64);
        let index = (ratio * (last_index as f64)).round() as usize;
        let value = ticks[index.min(last_index)];
        if sampled
            .last()
            .map(|prev| approx_equal(*prev, value))
            .unwrap_or(false)
        {
            continue;
        }
        sampled.push(value);
    }

    if sampled
        .first()
        .map(|first| !approx_equal(*first, ticks[0]))
        .unwrap_or(true)
    {
        sampled.insert(0, ticks[0]);
    }
    if sampled
        .last()
        .map(|last| !approx_equal(*last, ticks[last_index]))
        .unwrap_or(true)
    {
        sampled.push(ticks[last_index]);
    }

    for value in ticks {
        if sampled.len() >= target {
            break;
        }
        if sampled
            .iter()
            .any(|existing| approx_equal(*existing, value))
        {
            continue;
        }
        sampled.push(value);
    }

    sampled.sort_by(|lhs, rhs| lhs.total_cmp(rhs));
    sampled.dedup_by(|lhs, rhs| approx_equal(*lhs, *rhs));
    if sampled.len() > target {
        sampled.truncate(target);
    }
    sampled
}

fn approx_equal(lhs: f64, rhs: f64) -> bool {
    let scale = lhs.abs().max(rhs.abs()).max(1.0);
    (lhs - rhs).abs() <= scale * 1e-12
}
