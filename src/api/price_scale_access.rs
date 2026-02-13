use crate::core::{PriceScale, PriceScaleMode, PriceScaleTuning};
use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{
    ChartEngine, PriceScaleMarginBehavior, PriceScaleRealtimeBehavior,
    PriceScaleTransformedBaseBehavior, PriceScaleTransformedBaseSource,
};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn price_scale_realtime_behavior(&self) -> PriceScaleRealtimeBehavior {
        self.price_scale_realtime_behavior
    }

    pub fn set_price_scale_realtime_behavior(&mut self, behavior: PriceScaleRealtimeBehavior) {
        self.price_scale_realtime_behavior = behavior;
    }

    /// Maps a raw price value into pixel Y under the active price scale mode.
    pub fn map_price_to_pixel(&self, price: f64) -> ChartResult<f64> {
        self.price_scale.price_to_pixel(price, self.viewport)
    }

    /// Maps a pixel Y coordinate back into a raw price value.
    pub fn map_pixel_to_price(&self, pixel: f64) -> ChartResult<f64> {
        self.price_scale.pixel_to_price(pixel, self.viewport)
    }

    #[must_use]
    pub fn price_domain(&self) -> (f64, f64) {
        self.price_scale.domain()
    }

    /// Returns the active price scale mapping mode.
    #[must_use]
    pub fn price_scale_mode(&self) -> PriceScaleMode {
        self.price_scale_mode
    }

    /// Returns whether price-axis pixel mapping is inverted.
    #[must_use]
    pub fn price_scale_inverted(&self) -> bool {
        self.price_scale.is_inverted()
    }

    #[must_use]
    pub fn price_scale_transformed_base_behavior(&self) -> PriceScaleTransformedBaseBehavior {
        self.price_scale_transformed_base_behavior
    }

    /// Returns current resolved transformed base value when active mode is
    /// `Percentage` or `IndexedTo100`.
    #[must_use]
    pub fn price_scale_transformed_base_value(&self) -> Option<f64> {
        self.price_scale.base_value()
    }

    /// Enables/disables inverted price-axis mapping.
    pub fn set_price_scale_inverted(&mut self, inverted: bool) {
        self.price_scale = self.price_scale.with_inverted(inverted);
    }

    pub fn set_price_scale_transformed_base_behavior(
        &mut self,
        behavior: PriceScaleTransformedBaseBehavior,
    ) -> ChartResult<()> {
        if let Some(explicit) = behavior.explicit_base_price {
            if !explicit.is_finite() || explicit == 0.0 {
                return Err(ChartError::InvalidData(
                    "price scale transformed explicit base must be finite and non-zero".to_owned(),
                ));
            }
        }
        self.price_scale_transformed_base_behavior = behavior;
        let _ = self.refresh_price_scale_transformed_base()?;
        Ok(())
    }

    #[must_use]
    pub fn price_scale_margin_behavior(&self) -> PriceScaleMarginBehavior {
        let (top_margin_ratio, bottom_margin_ratio) = self.price_scale.margins();
        PriceScaleMarginBehavior {
            top_margin_ratio,
            bottom_margin_ratio,
        }
    }

    pub fn set_price_scale_margin_behavior(
        &mut self,
        behavior: PriceScaleMarginBehavior,
    ) -> ChartResult<()> {
        if !behavior.top_margin_ratio.is_finite()
            || !behavior.bottom_margin_ratio.is_finite()
            || behavior.top_margin_ratio < 0.0
            || behavior.bottom_margin_ratio < 0.0
        {
            return Err(ChartError::InvalidData(
                "price scale margins must be finite and >= 0".to_owned(),
            ));
        }
        if behavior.top_margin_ratio + behavior.bottom_margin_ratio >= 1.0 {
            return Err(ChartError::InvalidData(
                "price scale margins must sum to < 1".to_owned(),
            ));
        }
        self.price_scale = self
            .price_scale
            .with_margins(behavior.top_margin_ratio, behavior.bottom_margin_ratio)?;
        Ok(())
    }

    /// Switches the price scale mapping mode while preserving the current raw domain.
    ///
    /// When switching to `PriceScaleMode::Log`, the current domain must be strictly positive.
    /// Transformed display modes (`Percentage` / `IndexedTo100`) resolve a
    /// deterministic non-zero base value from the current domain.
    pub fn set_price_scale_mode(&mut self, mode: PriceScaleMode) -> ChartResult<()> {
        let base_value = self.resolve_price_scale_transformed_base_value(mode);
        self.price_scale = self.price_scale.with_mode_and_base(mode, base_value)?;
        self.price_scale_mode = mode;
        Ok(())
    }

    pub fn autoscale_price_from_data(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_data_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from points with explicit tuning.
    pub fn autoscale_price_from_data_tuned(&mut self, tuning: PriceScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        let base_value = self.resolve_price_scale_transformed_base_value(self.price_scale_mode);
        self.price_scale =
            PriceScale::from_data_tuned_with_mode(&self.points, tuning, self.price_scale_mode)?
                .with_base_value(base_value)?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }

    pub fn autoscale_price_from_candles(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_candles_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from candles with explicit tuning.
    pub fn autoscale_price_from_candles_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        if self.candles.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale =
            PriceScale::from_ohlc_tuned_with_mode(&self.candles, tuning, self.price_scale_mode)?
                .with_base_value(
                    self.resolve_price_scale_transformed_base_value(self.price_scale_mode),
                )?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }

    /// Autoscales price domain from currently visible points.
    pub fn autoscale_price_from_visible_data(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_visible_data_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from visible points with explicit tuning.
    pub fn autoscale_price_from_visible_data_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        let visible = self.visible_points();
        if visible.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale =
            PriceScale::from_data_tuned_with_mode(&visible, tuning, self.price_scale_mode)?
                .with_base_value(
                    self.resolve_price_scale_transformed_base_value(self.price_scale_mode),
                )?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }

    /// Autoscales price domain from currently visible candles.
    pub fn autoscale_price_from_visible_candles(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_visible_candles_tuned(PriceScaleTuning::default())
    }

    /// Autoscales price domain from visible candles with explicit tuning.
    pub fn autoscale_price_from_visible_candles_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        let visible = self.visible_candles();
        if visible.is_empty() {
            return Ok(());
        }
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale =
            PriceScale::from_ohlc_tuned_with_mode(&visible, tuning, self.price_scale_mode)?
                .with_base_value(
                    self.resolve_price_scale_transformed_base_value(self.price_scale_mode),
                )?
                .with_inverted(keep_inverted)
                .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }

    pub(crate) fn rebuild_price_scale_from_domain_preserving_mode(
        &mut self,
        domain_start: f64,
        domain_end: f64,
    ) -> ChartResult<()> {
        let keep_inverted = self.price_scale.is_inverted();
        let keep_margins = self.price_scale.margins();
        self.price_scale = PriceScale::new_with_mode_and_base(
            domain_start,
            domain_end,
            self.price_scale_mode,
            self.resolve_price_scale_transformed_base_value(self.price_scale_mode),
        )?
        .with_inverted(keep_inverted)
        .with_margins(keep_margins.0, keep_margins.1)?;
        Ok(())
    }

    pub(crate) fn refresh_price_scale_transformed_base(&mut self) -> ChartResult<bool> {
        if !matches!(
            self.price_scale_mode,
            PriceScaleMode::Percentage | PriceScaleMode::IndexedTo100
        ) {
            return Ok(false);
        }

        let current = self.price_scale.base_value();
        let target = self.resolve_price_scale_transformed_base_value(self.price_scale_mode);
        if option_price_eq(current, target) {
            return Ok(false);
        }
        self.price_scale = self.price_scale.with_base_value(target)?;
        Ok(true)
    }

    fn resolve_price_scale_transformed_base_value(&self, mode: PriceScaleMode) -> Option<f64> {
        if !matches!(
            mode,
            PriceScaleMode::Percentage | PriceScaleMode::IndexedTo100
        ) {
            return None;
        }

        if let Some(base) = self
            .price_scale_transformed_base_behavior
            .explicit_base_price
        {
            return Some(base);
        }

        let candidate = match self.price_scale_transformed_base_behavior.dynamic_source {
            PriceScaleTransformedBaseSource::DomainStart => None,
            PriceScaleTransformedBaseSource::FirstData => {
                resolve_data_extreme_price(&self.points, &self.candles, false, None)
            }
            PriceScaleTransformedBaseSource::LastData => {
                resolve_data_extreme_price(&self.points, &self.candles, true, None)
            }
            PriceScaleTransformedBaseSource::FirstVisibleData => resolve_data_extreme_price(
                &self.points,
                &self.candles,
                false,
                Some(self.time_scale.visible_range()),
            )
            .or_else(|| resolve_data_extreme_price(&self.points, &self.candles, false, None)),
            PriceScaleTransformedBaseSource::LastVisibleData => resolve_data_extreme_price(
                &self.points,
                &self.candles,
                true,
                Some(self.time_scale.visible_range()),
            )
            .or_else(|| resolve_data_extreme_price(&self.points, &self.candles, true, None)),
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
    points: &[crate::core::DataPoint],
    candles: &[crate::core::OhlcBar],
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
