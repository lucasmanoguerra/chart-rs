use crate::error::{ChartError, ChartResult};

use super::InteractionInputBehavior;

pub(super) fn validate_zoom_inputs(
    factor: f64,
    anchor_px: f64,
    min_span_absolute: f64,
) -> ChartResult<()> {
    if !factor.is_finite() || factor <= 0.0 {
        return Err(ChartError::InvalidData(
            "zoom factor must be finite and > 0".to_owned(),
        ));
    }
    if !anchor_px.is_finite() {
        return Err(ChartError::InvalidData(
            "zoom anchor px must be finite".to_owned(),
        ));
    }
    if !min_span_absolute.is_finite() || min_span_absolute <= 0.0 {
        return Err(ChartError::InvalidData(
            "zoom min span must be finite and > 0".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_pan_pixel_delta(delta_px: f64) -> ChartResult<()> {
    if !delta_px.is_finite() {
        return Err(ChartError::InvalidData(
            "pan pixel delta must be finite".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_touch_drag_deltas(
    behavior: InteractionInputBehavior,
    delta_x_px: f64,
    delta_y_px: f64,
) -> ChartResult<()> {
    if behavior.scroll_horz_touch_drag && !delta_x_px.is_finite() {
        return Err(ChartError::InvalidData(
            "touch drag horizontal delta must be finite when horizontal touch pan is enabled"
                .to_owned(),
        ));
    }
    if behavior.scroll_vert_touch_drag && !delta_y_px.is_finite() {
        return Err(ChartError::InvalidData(
            "touch drag vertical delta must be finite when vertical touch pan is enabled"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_wheel_pan_inputs(
    wheel_delta_x: f64,
    pan_step_ratio: f64,
) -> ChartResult<()> {
    if !wheel_delta_x.is_finite() {
        return Err(ChartError::InvalidData(
            "wheel pan delta must be finite".to_owned(),
        ));
    }
    if !pan_step_ratio.is_finite() || pan_step_ratio <= 0.0 {
        return Err(ChartError::InvalidData(
            "wheel pan step ratio must be finite and > 0".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_wheel_zoom_inputs(
    wheel_delta_y: f64,
    zoom_step_ratio: f64,
) -> ChartResult<()> {
    if !wheel_delta_y.is_finite() {
        return Err(ChartError::InvalidData(
            "wheel delta must be finite".to_owned(),
        ));
    }
    if !zoom_step_ratio.is_finite() || zoom_step_ratio <= 0.0 {
        return Err(ChartError::InvalidData(
            "wheel zoom step ratio must be finite and > 0".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_pan_pixel_delta, validate_wheel_pan_inputs, validate_wheel_zoom_inputs,
        validate_zoom_inputs,
    };

    #[test]
    fn zoom_input_validation_rejects_invalid_factor() {
        let err = validate_zoom_inputs(0.0, 10.0, 1.0).expect_err("factor must fail");
        assert!(format!("{err}").contains("zoom factor"));
    }

    #[test]
    fn pan_pixel_validation_rejects_non_finite() {
        let err = validate_pan_pixel_delta(f64::NAN).expect_err("nan pan delta must fail");
        assert!(format!("{err}").contains("pan pixel delta"));
    }

    #[test]
    fn wheel_pan_validation_rejects_non_positive_step_ratio() {
        let err = validate_wheel_pan_inputs(120.0, 0.0).expect_err("step ratio must fail");
        assert!(format!("{err}").contains("wheel pan step ratio"));
    }

    #[test]
    fn wheel_zoom_validation_rejects_non_positive_step_ratio() {
        let err = validate_wheel_zoom_inputs(120.0, -0.1).expect_err("step ratio must fail");
        assert!(format!("{err}").contains("wheel zoom step ratio"));
    }
}
