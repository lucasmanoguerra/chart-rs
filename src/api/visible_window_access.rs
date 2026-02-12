use crate::core::{DataPoint, OhlcBar, candles_in_time_window, points_in_time_window};
use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, expand_visible_window};

impl<R: Renderer> ChartEngine<R> {
    /// Returns point samples currently inside the visible time window.
    #[must_use]
    pub fn visible_points(&self) -> Vec<DataPoint> {
        let (start, end) = self.time_scale.visible_range();
        points_in_time_window(&self.points, start, end)
    }

    /// Returns candle samples currently inside the visible time window.
    #[must_use]
    pub fn visible_candles(&self) -> Vec<OhlcBar> {
        let (start, end) = self.time_scale.visible_range();
        candles_in_time_window(&self.candles, start, end)
    }

    /// Returns visible points with symmetric overscan around the visible window.
    pub fn visible_points_with_overscan(&self, ratio: f64) -> ChartResult<Vec<DataPoint>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        Ok(points_in_time_window(&self.points, start, end))
    }

    /// Returns visible candles with symmetric overscan around the visible window.
    pub fn visible_candles_with_overscan(&self, ratio: f64) -> ChartResult<Vec<OhlcBar>> {
        let (start, end) = expand_visible_window(self.time_scale.visible_range(), ratio)?;
        Ok(candles_in_time_window(&self.candles, start, end))
    }
}
