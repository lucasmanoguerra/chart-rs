use crate::render::CanvasLayerKind;

use super::{InvalidationLevel, InvalidationMask, InvalidationTopic};

pub(super) const CURSOR_ONLY_PLOT_LAYERS: [CanvasLayerKind; 3] = [
    CanvasLayerKind::Background,
    CanvasLayerKind::Overlay,
    CanvasLayerKind::Crosshair,
];

pub(super) const LIGHT_PLOT_LAYERS: [CanvasLayerKind; 5] = [
    CanvasLayerKind::Background,
    CanvasLayerKind::Grid,
    CanvasLayerKind::Series,
    CanvasLayerKind::Overlay,
    CanvasLayerKind::Crosshair,
];

pub(super) const AXIS_ONLY_PLOT_LAYERS: [CanvasLayerKind; 0] = [];

#[must_use]
pub(super) fn select_plot_layers_for_pending(
    pending: InvalidationMask,
) -> &'static [CanvasLayerKind] {
    if is_cursor_only_invalidation(pending) {
        return &CURSOR_ONLY_PLOT_LAYERS;
    }
    if is_axis_cursor_only_invalidation(pending) {
        return &CURSOR_ONLY_PLOT_LAYERS;
    }
    if is_axis_only_invalidation(pending) {
        return &AXIS_ONLY_PLOT_LAYERS;
    }
    &LIGHT_PLOT_LAYERS
}

#[must_use]
pub(super) fn pending_has_time_scale_topic(pending: InvalidationMask) -> bool {
    pending.has_topic(InvalidationTopic::TimeScale)
}

fn is_cursor_only_invalidation(pending: InvalidationMask) -> bool {
    pending.level() == InvalidationLevel::Cursor
}

fn is_axis_only_invalidation(pending: InvalidationMask) -> bool {
    pending.has_topic(InvalidationTopic::Axis)
        && !pending.has_topic(InvalidationTopic::General)
        && !pending.has_topic(InvalidationTopic::Cursor)
        && !pending.has_topic(InvalidationTopic::TimeScale)
        && !pending.has_topic(InvalidationTopic::PriceScale)
        && !pending.has_topic(InvalidationTopic::Series)
        && !pending.has_topic(InvalidationTopic::PaneLayout)
        && !pending.has_topic(InvalidationTopic::Style)
        && !pending.has_topic(InvalidationTopic::Plugin)
}

fn is_axis_cursor_only_invalidation(pending: InvalidationMask) -> bool {
    pending.has_topic(InvalidationTopic::Axis)
        && pending.has_topic(InvalidationTopic::Cursor)
        && !pending.has_topic(InvalidationTopic::General)
        && !pending.has_topic(InvalidationTopic::TimeScale)
        && !pending.has_topic(InvalidationTopic::PriceScale)
        && !pending.has_topic(InvalidationTopic::Series)
        && !pending.has_topic(InvalidationTopic::PaneLayout)
        && !pending.has_topic(InvalidationTopic::Style)
        && !pending.has_topic(InvalidationTopic::Plugin)
}

#[cfg(test)]
mod tests {
    use super::{
        AXIS_ONLY_PLOT_LAYERS, CURSOR_ONLY_PLOT_LAYERS, LIGHT_PLOT_LAYERS,
        pending_has_time_scale_topic, select_plot_layers_for_pending,
    };
    use crate::api::{InvalidationLevel, InvalidationMask, InvalidationTopic, InvalidationTopics};

    #[test]
    fn select_plot_layers_uses_cursor_only_for_pure_cursor_invalidations() {
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        );
        assert_eq!(
            select_plot_layers_for_pending(pending),
            &CURSOR_ONLY_PLOT_LAYERS
        );
    }

    #[test]
    fn select_plot_layers_uses_light_layers_for_series_light_invalidations() {
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );
        assert_eq!(select_plot_layers_for_pending(pending), &LIGHT_PLOT_LAYERS);
    }

    #[test]
    fn select_plot_layers_uses_cursor_layers_for_axis_cursor_light_invalidations() {
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Axis)
                .with_topic(InvalidationTopic::Cursor),
        );
        assert_eq!(
            select_plot_layers_for_pending(pending),
            &CURSOR_ONLY_PLOT_LAYERS
        );
    }

    #[test]
    fn select_plot_layers_uses_axis_only_layers_for_axis_only_invalidations() {
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Axis),
        );
        assert_eq!(
            select_plot_layers_for_pending(pending),
            &AXIS_ONLY_PLOT_LAYERS
        );
    }

    #[test]
    fn pending_has_time_scale_topic_detects_time_scale_presence() {
        let with_topic = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::TimeScale),
        );
        let without_topic = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );

        assert!(pending_has_time_scale_topic(with_topic));
        assert!(!pending_has_time_scale_topic(without_topic));
    }
}
