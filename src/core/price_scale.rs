use crate::core::{DataPoint, LinearScale, OhlcBar, Viewport};
use crate::error::{ChartError, ChartResult};

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceScale {
    linear: LinearScale,
}

impl PriceScale {
    pub fn new(price_min: f64, price_max: f64) -> ChartResult<Self> {
        let linear = LinearScale::new(price_min, price_max)?;
        Ok(Self { linear })
    }

    pub fn from_data(points: &[DataPoint]) -> ChartResult<Self> {
        Self::from_data_tuned(points, PriceScaleTuning::default())
    }

    pub fn from_data_tuned(points: &[DataPoint], tuning: PriceScaleTuning) -> ChartResult<Self> {
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

        Self::from_min_max_tuned(min, max, tuning)
    }

    pub fn from_ohlc(bars: &[OhlcBar]) -> ChartResult<Self> {
        Self::from_ohlc_tuned(bars, PriceScaleTuning::default())
    }

    pub fn from_ohlc_tuned(bars: &[OhlcBar], tuning: PriceScaleTuning) -> ChartResult<Self> {
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

        Self::from_min_max_tuned(min, max, tuning)
    }

    #[must_use]
    pub fn domain(self) -> (f64, f64) {
        self.linear.domain()
    }

    pub fn price_to_pixel(self, price: f64, viewport: Viewport) -> ChartResult<f64> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }

        let axis_viewport = Viewport::new(viewport.height, 1);
        let y_from_bottom = self.linear.domain_to_pixel(price, axis_viewport)?;
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
        self.linear.pixel_to_domain(y_from_bottom, axis_viewport)
    }

    fn from_min_max_tuned(min: f64, max: f64, tuning: PriceScaleTuning) -> ChartResult<Self> {
        let tuning = tuning.validate()?;
        let (base_min, base_max) = normalize_range(min, max, tuning.min_span_absolute)?;
        let span = base_max - base_min;

        let padded_min = base_min - span * tuning.bottom_padding_ratio;
        let padded_max = base_max + span * tuning.top_padding_ratio;
        let normalized = normalize_range(padded_min, padded_max, tuning.min_span_absolute)?;

        Self::new(normalized.0, normalized.1)
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
