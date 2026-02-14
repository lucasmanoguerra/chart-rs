use crate::error::{ChartError, ChartResult};

use super::{PriceScaleMarginBehavior, PriceScaleTransformedBaseBehavior};

pub(super) fn validate_price_scale_transformed_base_behavior(
    behavior: PriceScaleTransformedBaseBehavior,
) -> ChartResult<()> {
    if let Some(explicit) = behavior.explicit_base_price {
        if !explicit.is_finite() || explicit == 0.0 {
            return Err(ChartError::InvalidData(
                "price scale transformed explicit base must be finite and non-zero".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_price_scale_margin_behavior(
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
    Ok(())
}
