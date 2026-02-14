use crate::error::{ChartError, ChartResult};

const WHEEL_STEP_UNITS: f64 = 120.0;

pub(super) fn resolve_wheel_zoom_factor(
    wheel_delta_y: f64,
    zoom_step_ratio: f64,
) -> ChartResult<Option<f64>> {
    if wheel_delta_y == 0.0 {
        return Ok(None);
    }

    let normalized_steps = wheel_delta_y / WHEEL_STEP_UNITS;
    let base = 1.0 + zoom_step_ratio;
    let factor = base.powf(-normalized_steps);
    if !factor.is_finite() || factor <= 0.0 {
        return Err(ChartError::InvalidData(
            "computed wheel zoom factor must be finite and > 0".to_owned(),
        ));
    }
    Ok(Some(factor))
}

pub(super) fn resolve_pinch_zoom_factor(pinch_scale_factor: f64) -> ChartResult<Option<f64>> {
    if !pinch_scale_factor.is_finite() || pinch_scale_factor <= 0.0 {
        return Err(ChartError::InvalidData(
            "pinch zoom factor must be finite and > 0".to_owned(),
        ));
    }
    if (pinch_scale_factor - 1.0).abs() <= f64::EPSILON {
        return Ok(None);
    }
    Ok(Some(pinch_scale_factor))
}

#[cfg(test)]
mod tests {
    use super::{resolve_pinch_zoom_factor, resolve_wheel_zoom_factor};

    #[test]
    fn zero_wheel_delta_returns_none() {
        let factor = resolve_wheel_zoom_factor(0.0, 0.1).expect("factor");
        assert!(factor.is_none());
    }

    #[test]
    fn negative_wheel_delta_produces_zoom_in_factor() {
        let factor = resolve_wheel_zoom_factor(-120.0, 0.1)
            .expect("factor")
            .expect("some");
        assert!(factor > 1.0);
    }

    #[test]
    fn pinch_factor_resolver_returns_none_for_unity_factor() {
        let factor = resolve_pinch_zoom_factor(1.0).expect("factor");
        assert!(factor.is_none());
    }

    #[test]
    fn pinch_factor_resolver_rejects_non_positive_or_non_finite() {
        let err = resolve_pinch_zoom_factor(0.0).expect_err("zero must fail");
        assert!(format!("{err}").contains("pinch zoom factor"));

        let err = resolve_pinch_zoom_factor(f64::NAN).expect_err("nan must fail");
        assert!(format!("{err}").contains("pinch zoom factor"));
    }
}
