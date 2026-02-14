use crate::core::TimeIndexCoordinateSpace;
use crate::error::ChartResult;

pub(super) struct AnchorPreservingZoomTarget {
    pub(super) target_bar_spacing: f64,
    pub(super) target_right_offset: f64,
    pub(super) anchor_time_before: f64,
    pub(super) anchor_time_after: f64,
}

pub(super) struct AnchorPreservingZoomTargetInput {
    pub(super) space: TimeIndexCoordinateSpace,
    pub(super) reference_step: f64,
    pub(super) full_end: f64,
    pub(super) visible_start: f64,
    pub(super) visible_end: f64,
    pub(super) anchor_px: f64,
    pub(super) viewport_width: f64,
    pub(super) factor: f64,
    pub(super) min_span_absolute: f64,
    pub(super) anchor_time_before: f64,
}

impl AnchorPreservingZoomTarget {
    pub(super) fn anchor_time_drift_abs(&self) -> f64 {
        (self.anchor_time_after - self.anchor_time_before).abs()
    }
}

pub(super) fn resolve_anchor_preserving_zoom_target(
    input: AnchorPreservingZoomTargetInput,
) -> ChartResult<AnchorPreservingZoomTarget> {
    let AnchorPreservingZoomTargetInput {
        space,
        reference_step,
        full_end,
        visible_start,
        visible_end,
        anchor_px,
        viewport_width,
        factor,
        min_span_absolute,
        anchor_time_before,
    } = input;

    let current_span = (visible_end - visible_start).max(1e-9);
    let target_span = (current_span / factor).max(min_span_absolute);
    let effective_factor = current_span / target_span;
    let target_bar_spacing = (space.bar_spacing_px * effective_factor).max(f64::EPSILON);

    let anchor_x = anchor_px.clamp(0.0, viewport_width);
    let anchor_logical_index = space.coordinate_to_logical_index(anchor_x)?;
    let zoomed_space = TimeIndexCoordinateSpace {
        bar_spacing_px: target_bar_spacing,
        ..space
    };
    let target_right_offset = zoomed_space.solve_right_offset_for_anchor_preserving_zoom(
        space.bar_spacing_px,
        space.right_offset_bars,
        anchor_logical_index,
    )?;
    let target_end = full_end + target_right_offset * reference_step;
    let target_start = target_end - target_span;
    let anchor_time_after = if viewport_width > 0.0 {
        target_start + (anchor_x / viewport_width) * target_span
    } else {
        anchor_time_before
    };

    Ok(AnchorPreservingZoomTarget {
        target_bar_spacing,
        target_right_offset,
        anchor_time_before,
        anchor_time_after,
    })
}

#[cfg(test)]
mod tests {
    use super::{AnchorPreservingZoomTargetInput, resolve_anchor_preserving_zoom_target};
    use crate::core::TimeIndexCoordinateSpace;

    #[test]
    fn resolver_keeps_anchor_stable_for_basic_space() {
        let space = TimeIndexCoordinateSpace {
            base_index: 100.0,
            right_offset_bars: 0.0,
            bar_spacing_px: 6.0,
            width_px: 600.0,
        };

        let target = resolve_anchor_preserving_zoom_target(AnchorPreservingZoomTargetInput {
            space,
            reference_step: 1.0,
            full_end: 100.0,
            visible_start: 0.0,
            visible_end: 100.0,
            anchor_px: 300.0,
            viewport_width: 600.0,
            factor: 1.5,
            min_span_absolute: 1e-6,
            anchor_time_before: 50.0,
        })
        .expect("target");
        assert!(target.target_bar_spacing > 0.0);
        assert!(target.target_right_offset.is_finite());
        assert!(target.anchor_time_after.is_finite());
    }
}
