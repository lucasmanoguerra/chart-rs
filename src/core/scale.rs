use crate::core::types::Viewport;
use crate::error::{ChartError, ChartResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    domain_start: f64,
    domain_end: f64,
}

impl LinearScale {
    pub fn new(domain_start: f64, domain_end: f64) -> ChartResult<Self> {
        if !domain_start.is_finite() || !domain_end.is_finite() || domain_start == domain_end {
            return Err(ChartError::InvalidData(
                "scale domain must be finite and non-zero".to_owned(),
            ));
        }

        Ok(Self {
            domain_start,
            domain_end,
        })
    }

    #[must_use]
    pub fn domain(self) -> (f64, f64) {
        (self.domain_start, self.domain_end)
    }

    pub fn domain_to_pixel(self, value: f64, viewport: Viewport) -> ChartResult<f64> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }

        if !value.is_finite() {
            return Err(ChartError::InvalidData("value must be finite".to_owned()));
        }

        let span = self.domain_end - self.domain_start;
        let normalized = (value - self.domain_start) / span;
        Ok(normalized * f64::from(viewport.width))
    }

    pub fn pixel_to_domain(self, pixel: f64, viewport: Viewport) -> ChartResult<f64> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }

        if !pixel.is_finite() {
            return Err(ChartError::InvalidData("pixel must be finite".to_owned()));
        }

        let span = self.domain_end - self.domain_start;
        let normalized = pixel / f64::from(viewport.width);
        Ok(self.domain_start + normalized * span)
    }
}
