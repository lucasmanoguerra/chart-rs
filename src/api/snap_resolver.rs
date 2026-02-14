use ordered_float::OrderedFloat;
use smallvec::SmallVec;

use crate::interaction::CrosshairSnap;
use crate::render::Renderer;

use super::ChartEngine;

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn snap_at_x(&self, pointer_x: f64) -> Option<CrosshairSnap> {
        let mut candidates: SmallVec<[(OrderedFloat<f64>, CrosshairSnap); 2]> = SmallVec::new();
        if let Some(snap) = self.nearest_data_snap(pointer_x) {
            candidates.push(snap);
        }
        if let Some(snap) = self.nearest_candle_snap(pointer_x) {
            candidates.push(snap);
        }

        candidates
            .into_iter()
            .min_by_key(|item| item.0)
            .map(|(_, snap)| snap)
    }

    fn nearest_data_snap(&self, pointer_x: f64) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        match (
            self.nearest_data_snap_sparse(pointer_x),
            self.nearest_data_snap_bruteforce(pointer_x),
        ) {
            (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        }
    }

    fn nearest_data_snap_sparse(
        &self,
        pointer_x: f64,
    ) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        let (space, reference_step) = self.resolve_time_index_coordinate_space()?;
        let slot = space
            .coordinate_to_nearest_filled_slot(pointer_x, self.core.model.points.len(), |idx| {
                self.core.model.points[idx].x / reference_step
            })
            .ok()??;
        let point = self.core.model.points.get(slot)?;
        let x_px = self
            .core
            .model
            .time_scale
            .time_to_pixel(point.x, self.core.model.viewport)
            .ok()?;
        let y_px = self
            .core
            .model
            .price_scale
            .price_to_pixel(point.y, self.core.model.viewport)
            .ok()?;
        let dist = OrderedFloat((x_px - pointer_x).abs());
        Some((
            dist,
            CrosshairSnap {
                x: x_px,
                y: y_px,
                time: point.x,
                price: point.y,
            },
        ))
    }

    fn nearest_data_snap_bruteforce(
        &self,
        pointer_x: f64,
    ) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        let mut best: Option<(OrderedFloat<f64>, CrosshairSnap)> = None;
        for point in &self.core.model.points {
            let x_px = match self
                .core
                .model
                .time_scale
                .time_to_pixel(point.x, self.core.model.viewport)
            {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y_px = match self
                .core
                .model
                .price_scale
                .price_to_pixel(point.y, self.core.model.viewport)
            {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dist = OrderedFloat((x_px - pointer_x).abs());
            match best {
                Some((current, _)) if current <= dist => {}
                _ => {
                    best = Some((
                        dist,
                        CrosshairSnap {
                            x: x_px,
                            y: y_px,
                            time: point.x,
                            price: point.y,
                        },
                    ))
                }
            }
        }
        best
    }

    fn nearest_candle_snap(&self, pointer_x: f64) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        match (
            self.nearest_candle_snap_sparse(pointer_x),
            self.nearest_candle_snap_bruteforce(pointer_x),
        ) {
            (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        }
    }

    fn nearest_candle_snap_sparse(
        &self,
        pointer_x: f64,
    ) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        let (space, reference_step) = self.resolve_time_index_coordinate_space()?;
        let slot = space
            .coordinate_to_nearest_filled_slot(pointer_x, self.core.model.candles.len(), |idx| {
                self.core.model.candles[idx].time / reference_step
            })
            .ok()??;
        let candle = self.core.model.candles.get(slot)?;
        let x_px = self
            .core
            .model
            .time_scale
            .time_to_pixel(candle.time, self.core.model.viewport)
            .ok()?;
        let y_px = self
            .core
            .model
            .price_scale
            .price_to_pixel(candle.close, self.core.model.viewport)
            .ok()?;
        let dist = OrderedFloat((x_px - pointer_x).abs());
        Some((
            dist,
            CrosshairSnap {
                x: x_px,
                y: y_px,
                time: candle.time,
                price: candle.close,
            },
        ))
    }

    fn nearest_candle_snap_bruteforce(
        &self,
        pointer_x: f64,
    ) -> Option<(OrderedFloat<f64>, CrosshairSnap)> {
        let mut best: Option<(OrderedFloat<f64>, CrosshairSnap)> = None;
        for candle in &self.core.model.candles {
            let x_px = match self
                .core
                .model
                .time_scale
                .time_to_pixel(candle.time, self.core.model.viewport)
            {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y_px = match self
                .core
                .model
                .price_scale
                .price_to_pixel(candle.close, self.core.model.viewport)
            {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dist = OrderedFloat((x_px - pointer_x).abs());
            match best {
                Some((current, _)) if current <= dist => {}
                _ => {
                    best = Some((
                        dist,
                        CrosshairSnap {
                            x: x_px,
                            y: y_px,
                            time: candle.time,
                            price: candle.close,
                        },
                    ))
                }
            }
        }
        best
    }
}
