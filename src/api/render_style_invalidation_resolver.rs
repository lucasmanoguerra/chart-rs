use super::{InvalidationTopic, InvalidationTopics, RenderStyle};

const RENDER_STYLE_LIGHT_TOPICS: InvalidationTopics =
    InvalidationTopics::from_topic(InvalidationTopic::Style)
        .with_topic(InvalidationTopic::Axis)
        .with_topic(InvalidationTopic::Series)
        .with_topic(InvalidationTopic::Cursor);

pub(super) enum RenderStyleInvalidationDecision {
    None,
    Full,
    Light(InvalidationTopics),
}

pub(super) fn resolve_render_style_invalidation(
    previous: RenderStyle,
    next: RenderStyle,
) -> RenderStyleInvalidationDecision {
    if previous == next {
        return RenderStyleInvalidationDecision::None;
    }
    if render_style_layout_changed(previous, next) {
        return RenderStyleInvalidationDecision::Full;
    }
    RenderStyleInvalidationDecision::Light(RENDER_STYLE_LIGHT_TOPICS)
}

fn render_style_layout_changed(previous: RenderStyle, next: RenderStyle) -> bool {
    previous.price_axis_width_px != next.price_axis_width_px
        || previous.time_axis_height_px != next.time_axis_height_px
}

#[cfg(test)]
mod tests {
    use super::{RenderStyleInvalidationDecision, resolve_render_style_invalidation};
    use crate::api::{InvalidationTopic, RenderStyle};

    #[test]
    fn resolver_returns_none_when_style_is_identical() {
        let style = RenderStyle::default();
        let decision = resolve_render_style_invalidation(style, style);
        assert!(matches!(decision, RenderStyleInvalidationDecision::None));
    }

    #[test]
    fn resolver_returns_full_when_layout_changes() {
        let previous = RenderStyle::default();
        let next = RenderStyle {
            price_axis_width_px: previous.price_axis_width_px + 5.0,
            ..previous
        };

        let decision = resolve_render_style_invalidation(previous, next);
        assert!(matches!(decision, RenderStyleInvalidationDecision::Full));
    }

    #[test]
    fn resolver_returns_light_topics_for_non_layout_changes() {
        let previous = RenderStyle::default();
        let next = RenderStyle {
            show_price_axis_labels: !previous.show_price_axis_labels,
            ..previous
        };

        let decision = resolve_render_style_invalidation(previous, next);
        let RenderStyleInvalidationDecision::Light(topics) = decision else {
            panic!("expected light invalidation");
        };
        assert!(topics.contains_topic(InvalidationTopic::Style));
        assert!(topics.contains_topic(InvalidationTopic::Axis));
        assert!(topics.contains_topic(InvalidationTopic::Series));
        assert!(topics.contains_topic(InvalidationTopic::Cursor));
        assert!(!topics.contains_topic(InvalidationTopic::General));
        assert!(!topics.contains_topic(InvalidationTopic::TimeScale));
        assert!(!topics.contains_topic(InvalidationTopic::PriceScale));
        assert!(!topics.contains_topic(InvalidationTopic::PaneLayout));
        assert!(!topics.contains_topic(InvalidationTopic::Plugin));
    }
}
