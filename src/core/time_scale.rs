use crate::core::{DataPoint, LinearScale, Viewport};
use crate::error::{ChartError, ChartResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScale {
    linear: LinearScale,
}

impl TimeScale {
    pub fn new(time_start: f64, time_end: f64) -> ChartResult<Self> {
        let linear = LinearScale::new(time_start, time_end)?;
        Ok(Self { linear })
    }

    pub fn from_data(points: &[DataPoint]) -> ChartResult<Self> {
        if points.is_empty() {
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

    pub fn time_to_pixel(self, time: f64, viewport: Viewport) -> ChartResult<f64> {
        self.linear.domain_to_pixel(time, viewport)
    }

    pub fn pixel_to_time(self, pixel: f64, viewport: Viewport) -> ChartResult<f64> {
        self.linear.pixel_to_domain(pixel, viewport)
    }
}
