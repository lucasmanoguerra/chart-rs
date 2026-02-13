use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{
    ChartEngine, TimeCoordinateIndexPolicy, TimeFilledLogicalSlot, TimeFilledLogicalSource,
};

impl<R: Renderer> ChartEngine<R> {
    pub fn map_x_to_pixel(&self, x: f64) -> ChartResult<f64> {
        self.time_scale.time_to_pixel(x, self.viewport)
    }

    pub fn map_pixel_to_x(&self, pixel: f64) -> ChartResult<f64> {
        self.time_scale.pixel_to_time(pixel, self.viewport)
    }

    /// Maps pixel X to a floating logical bar index.
    ///
    /// `AllowWhitespace` returns direct logical index values, while
    /// `IgnoreWhitespace` resolves to nearest filled slot when data is sparse.
    pub fn map_pixel_to_logical_index(
        &self,
        pixel: f64,
        policy: TimeCoordinateIndexPolicy,
    ) -> ChartResult<Option<f64>> {
        if !pixel.is_finite() {
            return Err(ChartError::InvalidData(
                "logical-index pixel coordinate must be finite".to_owned(),
            ));
        }

        let Some((space, reference_step)) = self.resolve_time_index_coordinate_space() else {
            return Ok(None);
        };

        match policy {
            TimeCoordinateIndexPolicy::AllowWhitespace => {
                Ok(Some(space.coordinate_to_logical_index(pixel)?))
            }
            TimeCoordinateIndexPolicy::IgnoreWhitespace => {
                let mut best: Option<(f64, f64)> = None;

                if let Some(slot) =
                    space.coordinate_to_nearest_filled_slot(pixel, self.points.len(), |idx| {
                        self.points[idx].x / reference_step
                    })?
                {
                    let logical = self.points[slot].x / reference_step;
                    let candidate_x = space.index_to_coordinate(logical)?;
                    let distance = (candidate_x - pixel).abs();
                    best = Some((distance, logical));
                }

                if let Some(slot) =
                    space.coordinate_to_nearest_filled_slot(pixel, self.candles.len(), |idx| {
                        self.candles[idx].time / reference_step
                    })?
                {
                    let logical = self.candles[slot].time / reference_step;
                    let candidate_x = space.index_to_coordinate(logical)?;
                    let distance = (candidate_x - pixel).abs();
                    match best {
                        Some((best_distance, _)) if best_distance <= distance => {}
                        _ => {
                            best = Some((distance, logical));
                        }
                    }
                }

                Ok(best.map(|(_, logical)| logical))
            }
        }
    }

    /// Maps floating logical bar index to pixel X.
    pub fn map_logical_index_to_pixel(&self, logical_index: f64) -> ChartResult<Option<f64>> {
        if !logical_index.is_finite() {
            return Err(ChartError::InvalidData(
                "logical index must be finite".to_owned(),
            ));
        }
        let Some((space, _reference_step)) = self.resolve_time_index_coordinate_space() else {
            return Ok(None);
        };
        Ok(Some(space.index_to_coordinate(logical_index)?))
    }

    /// Maps pixel X to discrete logical index using ceil semantics.
    ///
    /// `AllowWhitespace` mirrors direct ceil conversion from floating logical
    /// indices. `IgnoreWhitespace` first resolves nearest filled logical slot.
    pub fn map_pixel_to_logical_index_ceil(
        &self,
        pixel: f64,
        policy: TimeCoordinateIndexPolicy,
    ) -> ChartResult<Option<i64>> {
        if !pixel.is_finite() {
            return Err(ChartError::InvalidData(
                "logical-index pixel coordinate must be finite".to_owned(),
            ));
        }

        let Some((space, _reference_step)) = self.resolve_time_index_coordinate_space() else {
            return Ok(None);
        };

        match policy {
            TimeCoordinateIndexPolicy::AllowWhitespace => {
                Ok(Some(space.coordinate_to_index_ceil(pixel)?))
            }
            TimeCoordinateIndexPolicy::IgnoreWhitespace => {
                let Some(logical) = self.map_pixel_to_logical_index(pixel, policy)? else {
                    return Ok(None);
                };
                if logical < (i64::MIN as f64) || logical > (i64::MAX as f64) {
                    return Err(ChartError::InvalidData(
                        "time logical index exceeds i64 range".to_owned(),
                    ));
                }
                Ok(Some(logical.ceil() as i64))
            }
        }
    }

    /// Resolves nearest filled sparse slot for a pixel coordinate.
    pub fn nearest_filled_logical_slot_at_pixel(
        &self,
        pixel: f64,
    ) -> ChartResult<Option<TimeFilledLogicalSlot>> {
        if !pixel.is_finite() {
            return Err(ChartError::InvalidData(
                "logical-index pixel coordinate must be finite".to_owned(),
            ));
        }
        let Some((space, reference_step)) = self.resolve_time_index_coordinate_space() else {
            return Ok(None);
        };

        let mut best: Option<(f64, TimeFilledLogicalSlot)> = None;

        if let Some(slot) =
            space.coordinate_to_nearest_filled_slot(pixel, self.points.len(), |idx| {
                self.points[idx].x / reference_step
            })?
        {
            let logical_index = self.points[slot].x / reference_step;
            let candidate_x = space.index_to_coordinate(logical_index)?;
            let distance = (candidate_x - pixel).abs();
            best = Some((
                distance,
                TimeFilledLogicalSlot {
                    source: TimeFilledLogicalSource::Points,
                    slot,
                    logical_index,
                    time: self.points[slot].x,
                },
            ));
        }

        if let Some(slot) =
            space.coordinate_to_nearest_filled_slot(pixel, self.candles.len(), |idx| {
                self.candles[idx].time / reference_step
            })?
        {
            let logical_index = self.candles[slot].time / reference_step;
            let candidate_x = space.index_to_coordinate(logical_index)?;
            let distance = (candidate_x - pixel).abs();
            let candidate = TimeFilledLogicalSlot {
                source: TimeFilledLogicalSource::Candles,
                slot,
                logical_index,
                time: self.candles[slot].time,
            };
            best = match best {
                Some((best_distance, best_slot))
                    if best_distance + 1e-12 < distance
                        || ((best_distance - distance).abs() <= 1e-12
                            && !matches!(best_slot.source, TimeFilledLogicalSource::Points)) =>
                {
                    Some((best_distance, best_slot))
                }
                _ => Some((distance, candidate)),
            };
        }

        Ok(best.map(|(_, slot)| slot))
    }

    /// Returns next filled logical index strictly greater than `logical_index`.
    pub fn next_filled_logical_index(&self, logical_index: f64) -> ChartResult<Option<f64>> {
        if !logical_index.is_finite() {
            return Err(ChartError::InvalidData(
                "logical index must be finite".to_owned(),
            ));
        }
        let Some((_, reference_step)) = self.resolve_time_index_coordinate_space() else {
            return Ok(None);
        };
        let filled = self.collect_unique_filled_logical_indices(reference_step);
        Ok(filled
            .into_iter()
            .find(|value| *value > logical_index + 1e-12))
    }

    /// Returns previous filled logical index strictly smaller than `logical_index`.
    pub fn prev_filled_logical_index(&self, logical_index: f64) -> ChartResult<Option<f64>> {
        if !logical_index.is_finite() {
            return Err(ChartError::InvalidData(
                "logical index must be finite".to_owned(),
            ));
        }
        let Some((_, reference_step)) = self.resolve_time_index_coordinate_space() else {
            return Ok(None);
        };
        let filled = self.collect_unique_filled_logical_indices(reference_step);
        Ok(filled
            .into_iter()
            .rev()
            .find(|value| *value < logical_index - 1e-12))
    }

    #[must_use]
    pub fn time_visible_range(&self) -> (f64, f64) {
        self.time_scale.visible_range()
    }

    #[must_use]
    pub fn time_full_range(&self) -> (f64, f64) {
        self.time_scale.full_range()
    }

    fn collect_unique_filled_logical_indices(&self, reference_step: f64) -> Vec<f64> {
        let mut indices = Vec::with_capacity(self.points.len() + self.candles.len());
        indices.extend(
            self.points
                .iter()
                .map(|point| point.x / reference_step)
                .filter(|value| value.is_finite()),
        );
        indices.extend(
            self.candles
                .iter()
                .map(|candle| candle.time / reference_step)
                .filter(|value| value.is_finite()),
        );

        indices.sort_by(|lhs, rhs| lhs.total_cmp(rhs));
        indices.dedup_by(|lhs, rhs| (*lhs - *rhs).abs() <= 1e-12);
        indices
    }
}
