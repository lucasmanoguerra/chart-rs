use crate::error::{ChartError, ChartResult};

const WHEEL_STEP_UNITS: f64 = 120.0;

pub(super) fn resolve_pixel_pan_delta_time(
    delta_px: f64,
    viewport_width_px: f64,
    visible_span: f64,
) -> ChartResult<f64> {
    if !viewport_width_px.is_finite() || viewport_width_px <= 0.0 {
        return Err(ChartError::InvalidData(
            "pan viewport width must be finite and > 0".to_owned(),
        ));
    }
    let delta_time = -(delta_px / viewport_width_px) * visible_span;
    if !delta_time.is_finite() {
        return Err(ChartError::InvalidData(
            "computed pixel pan delta time must be finite".to_owned(),
        ));
    }
    Ok(delta_time)
}

pub(super) fn resolve_touch_drag_pan_delta_time(
    delta_x_px: f64,
    delta_y_px: f64,
    viewport_width_px: f64,
    viewport_height_px: f64,
    visible_span: f64,
    scroll_horz_touch_drag: bool,
    scroll_vert_touch_drag: bool,
) -> ChartResult<Option<f64>> {
    let (driving_px, driving_axis_span_px) = match (scroll_horz_touch_drag, scroll_vert_touch_drag)
    {
        (true, false) => (delta_x_px, viewport_width_px),
        (false, true) => (delta_y_px, viewport_height_px),
        (true, true) => {
            if delta_x_px.abs() >= delta_y_px.abs() {
                (delta_x_px, viewport_width_px)
            } else {
                (delta_y_px, viewport_height_px)
            }
        }
        (false, false) => return Ok(None),
    };

    if driving_px == 0.0 {
        return Ok(None);
    }
    if !driving_axis_span_px.is_finite() || driving_axis_span_px <= 0.0 {
        return Err(ChartError::InvalidData(
            "touch drag driving axis span must be finite and > 0".to_owned(),
        ));
    }

    let delta_time = -(driving_px / driving_axis_span_px) * visible_span;
    if !delta_time.is_finite() {
        return Err(ChartError::InvalidData(
            "computed touch drag delta time must be finite".to_owned(),
        ));
    }
    Ok(Some(delta_time))
}

pub(super) fn resolve_wheel_pan_delta_time(
    wheel_delta_x: f64,
    visible_span: f64,
    pan_step_ratio: f64,
) -> ChartResult<Option<f64>> {
    if wheel_delta_x == 0.0 {
        return Ok(None);
    }

    let normalized_steps = wheel_delta_x / WHEEL_STEP_UNITS;
    let delta_time = normalized_steps * visible_span * pan_step_ratio;
    if !delta_time.is_finite() {
        return Err(ChartError::InvalidData(
            "computed wheel pan delta time must be finite".to_owned(),
        ));
    }
    Ok(Some(delta_time))
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_pixel_pan_delta_time, resolve_touch_drag_pan_delta_time,
        resolve_wheel_pan_delta_time,
    };

    #[test]
    fn pixel_pan_delta_matches_lwc_formula() {
        let delta = resolve_pixel_pan_delta_time(100.0, 500.0, 50.0).expect("delta");
        assert!((delta + 10.0).abs() <= 1e-12);
    }

    #[test]
    fn touch_drag_prefers_dominant_axis_when_both_enabled() {
        let delta = resolve_touch_drag_pan_delta_time(80.0, 20.0, 400.0, 200.0, 100.0, true, true)
            .expect("delta")
            .expect("some");
        assert!((delta + 20.0).abs() <= 1e-12);
    }

    #[test]
    fn touch_drag_returns_none_when_disabled_or_zero() {
        let none = resolve_touch_drag_pan_delta_time(10.0, 5.0, 400.0, 200.0, 100.0, false, false)
            .expect("none");
        assert!(none.is_none());

        let none = resolve_touch_drag_pan_delta_time(0.0, 5.0, 400.0, 200.0, 100.0, true, false)
            .expect("none");
        assert!(none.is_none());
    }

    #[test]
    fn zero_wheel_delta_returns_none() {
        let delta = resolve_wheel_pan_delta_time(0.0, 100.0, 0.2).expect("delta");
        assert!(delta.is_none());
    }

    #[test]
    fn wheel_pan_delta_matches_lwc_formula() {
        let delta = resolve_wheel_pan_delta_time(120.0, 50.0, 0.2)
            .expect("delta")
            .expect("some");
        assert!((delta - 10.0).abs() <= 1e-12);
    }

    #[test]
    fn wheel_pan_delta_rejects_non_finite_result() {
        let err = resolve_wheel_pan_delta_time(120.0, f64::INFINITY, 0.2).expect_err("finite");
        assert!(format!("{err}").contains("wheel pan delta time"));
    }
}
