use crate::core::{DataPoint, OhlcBar, PriceScale, PriceScaleMode};
use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, PriceScaleTransformedBaseSource};

pub(super) struct PriceScaleCoordinator;

impl PriceScaleCoordinator {
    pub(super) fn rebuild_price_scale_from_domain_preserving_mode<R: Renderer>(
        engine: &mut ChartEngine<R>,
        domain_start: f64,
        domain_end: f64,
    ) -> ChartResult<()> {
        let keep_inverted = engine.core.model.price_scale.is_inverted();
        let keep_margins = engine.core.model.price_scale.margins();
        engine.core.model.price_scale = PriceScale::new_with_mode_and_base(
            domain_start,
            domain_end,
            engine.core.model.price_scale_mode,
            Self::resolve_price_scale_transformed_base_value(
                engine,
                engine.core.model.price_scale_mode,
            ),
        )?
        .with_inverted(keep_inverted)
        .with_margins(keep_margins.0, keep_margins.1)?;
        engine.invalidate_price_scale();
        Ok(())
    }

    pub(super) fn refresh_price_scale_transformed_base<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        if !matches!(
            engine.core.model.price_scale_mode,
            PriceScaleMode::Percentage | PriceScaleMode::IndexedTo100
        ) {
            return Ok(false);
        }

        let current = engine.core.model.price_scale.base_value();
        let target = Self::resolve_price_scale_transformed_base_value(
            engine,
            engine.core.model.price_scale_mode,
        );
        if option_price_eq(current, target) {
            return Ok(false);
        }
        engine.core.model.price_scale = engine.core.model.price_scale.with_base_value(target)?;
        engine.invalidate_price_scale();
        Ok(true)
    }

    pub(super) fn resolve_price_scale_transformed_base_value<R: Renderer>(
        engine: &ChartEngine<R>,
        mode: PriceScaleMode,
    ) -> Option<f64> {
        if !matches!(
            mode,
            PriceScaleMode::Percentage | PriceScaleMode::IndexedTo100
        ) {
            return None;
        }

        if let Some(base) = engine
            .core
            .behavior
            .price_scale_transformed_base_behavior
            .explicit_base_price
        {
            return Some(base);
        }

        let candidate = match engine
            .core
            .behavior
            .price_scale_transformed_base_behavior
            .dynamic_source
        {
            PriceScaleTransformedBaseSource::DomainStart => None,
            PriceScaleTransformedBaseSource::FirstData => resolve_data_extreme_price(
                &engine.core.model.points,
                &engine.core.model.candles,
                false,
                None,
            ),
            PriceScaleTransformedBaseSource::LastData => resolve_data_extreme_price(
                &engine.core.model.points,
                &engine.core.model.candles,
                true,
                None,
            ),
            PriceScaleTransformedBaseSource::FirstVisibleData => resolve_data_extreme_price(
                &engine.core.model.points,
                &engine.core.model.candles,
                false,
                Some(engine.core.model.time_scale.visible_range()),
            )
            .or_else(|| {
                resolve_data_extreme_price(
                    &engine.core.model.points,
                    &engine.core.model.candles,
                    false,
                    None,
                )
            }),
            PriceScaleTransformedBaseSource::LastVisibleData => resolve_data_extreme_price(
                &engine.core.model.points,
                &engine.core.model.candles,
                true,
                Some(engine.core.model.time_scale.visible_range()),
            )
            .or_else(|| {
                resolve_data_extreme_price(
                    &engine.core.model.points,
                    &engine.core.model.candles,
                    true,
                    None,
                )
            }),
        };

        candidate.filter(|base| base.is_finite() && *base != 0.0)
    }
}

fn option_price_eq(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(lhs), Some(rhs)) => (lhs - rhs).abs() <= 1e-12,
        (None, None) => true,
        _ => false,
    }
}

fn resolve_data_extreme_price(
    points: &[DataPoint],
    candles: &[OhlcBar],
    pick_last: bool,
    visible_range: Option<(f64, f64)>,
) -> Option<f64> {
    let point_candidate = if pick_last {
        points
            .iter()
            .rev()
            .find(|point| is_inside_visible_range(point.x, visible_range))
            .map(|point| PriceBaseCandidate {
                time: point.x,
                price: point.y,
                source: PriceBaseCandidateSource::Points,
            })
    } else {
        points
            .iter()
            .find(|point| is_inside_visible_range(point.x, visible_range))
            .map(|point| PriceBaseCandidate {
                time: point.x,
                price: point.y,
                source: PriceBaseCandidateSource::Points,
            })
    };
    let candle_candidate = if pick_last {
        candles
            .iter()
            .rev()
            .find(|candle| is_inside_visible_range(candle.time, visible_range))
            .map(|candle| PriceBaseCandidate {
                time: candle.time,
                price: candle.close,
                source: PriceBaseCandidateSource::Candles,
            })
    } else {
        candles
            .iter()
            .find(|candle| is_inside_visible_range(candle.time, visible_range))
            .map(|candle| PriceBaseCandidate {
                time: candle.time,
                price: candle.close,
                source: PriceBaseCandidateSource::Candles,
            })
    };

    let selected = select_price_base_candidate(point_candidate, candle_candidate, pick_last)?;

    if !selected.price.is_finite() || selected.price == 0.0 {
        return None;
    }
    Some(selected.price)
}

fn is_inside_visible_range(time: f64, visible_range: Option<(f64, f64)>) -> bool {
    match visible_range {
        Some((start, end)) => time >= start && time <= end,
        None => true,
    }
}

#[derive(Clone, Copy)]
struct PriceBaseCandidate {
    time: f64,
    price: f64,
    source: PriceBaseCandidateSource,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PriceBaseCandidateSource {
    Points,
    Candles,
}

fn select_price_base_candidate(
    point: Option<PriceBaseCandidate>,
    candle: Option<PriceBaseCandidate>,
    pick_last: bool,
) -> Option<PriceBaseCandidate> {
    match (point, candle) {
        (Some(left), Some(right)) => {
            if pick_last {
                if left.time > right.time {
                    Some(left)
                } else if right.time > left.time {
                    Some(right)
                } else {
                    Some(prefer_candle_candidate(left, right))
                }
            } else if left.time < right.time {
                Some(left)
            } else if right.time < left.time {
                Some(right)
            } else {
                Some(prefer_candle_candidate(left, right))
            }
        }
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn prefer_candle_candidate(
    left: PriceBaseCandidate,
    right: PriceBaseCandidate,
) -> PriceBaseCandidate {
    if matches!(left.source, PriceBaseCandidateSource::Candles) {
        left
    } else if matches!(right.source, PriceBaseCandidateSource::Candles) {
        right
    } else {
        left
    }
}
