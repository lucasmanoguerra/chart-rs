use crate::core::{DataPoint, LinearScale, Viewport};
use crate::error::{ChartError, ChartResult};

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

        if min == max {
            let pad = if min == 0.0 { 1.0 } else { min.abs() * 0.05 };
            min -= pad;
            max += pad;
        }

        Self::new(min, max)
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
}
