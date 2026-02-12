use crate::error::{ChartError, ChartResult};
use crate::interaction::KineticPanConfig;

pub(super) fn validate_kinetic_pan_config(
    config: KineticPanConfig,
) -> ChartResult<KineticPanConfig> {
    if !config.decay_per_second.is_finite()
        || config.decay_per_second <= 0.0
        || config.decay_per_second >= 1.0
    {
        return Err(ChartError::InvalidData(
            "kinetic pan decay_per_second must be finite and in (0, 1)".to_owned(),
        ));
    }
    if !config.stop_velocity_abs.is_finite() || config.stop_velocity_abs <= 0.0 {
        return Err(ChartError::InvalidData(
            "kinetic pan stop_velocity_abs must be finite and > 0".to_owned(),
        ));
    }
    Ok(config)
}
