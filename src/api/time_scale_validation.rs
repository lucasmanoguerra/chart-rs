use crate::error::{ChartError, ChartResult};

use super::{
    TimeScaleNavigationBehavior, TimeScaleRealtimeAppendBehavior, TimeScaleZoomLimitBehavior,
};

pub(super) fn validate_time_scale_navigation_behavior(
    behavior: TimeScaleNavigationBehavior,
) -> ChartResult<()> {
    if !behavior.right_offset_bars.is_finite() {
        return Err(ChartError::InvalidData(
            "time scale right offset must be finite".to_owned(),
        ));
    }

    if let Some(bar_spacing_px) = behavior.bar_spacing_px {
        if !bar_spacing_px.is_finite() || bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale bar spacing must be finite and > 0".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_time_scale_realtime_append_behavior(
    behavior: TimeScaleRealtimeAppendBehavior,
) -> ChartResult<()> {
    if !behavior.right_edge_tolerance_bars.is_finite() || behavior.right_edge_tolerance_bars < 0.0 {
        return Err(ChartError::InvalidData(
            "time scale realtime right-edge tolerance must be finite and >= 0".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_time_scale_zoom_limit_behavior(
    behavior: TimeScaleZoomLimitBehavior,
) -> ChartResult<()> {
    if !behavior.min_bar_spacing_px.is_finite() || behavior.min_bar_spacing_px <= 0.0 {
        return Err(ChartError::InvalidData(
            "time scale minimum bar spacing must be finite and > 0".to_owned(),
        ));
    }

    if let Some(max_bar_spacing_px) = behavior.max_bar_spacing_px {
        if !max_bar_spacing_px.is_finite() || max_bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale maximum bar spacing must be finite and > 0".to_owned(),
            ));
        }
        if max_bar_spacing_px < behavior.min_bar_spacing_px {
            return Err(ChartError::InvalidData(
                "time scale maximum bar spacing must be >= minimum bar spacing".to_owned(),
            ));
        }
    }

    Ok(())
}
