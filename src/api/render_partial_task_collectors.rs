use crate::core::PaneId;
use crate::render::{CanvasLayerKind, LayeredRenderFrame};

use super::render_partial_plan::PartialCairoRenderPlan;
use super::render_partial_task::PanePartialRenderTask;

const AXIS_LAYERS: [CanvasLayerKind; 1] = [CanvasLayerKind::Axis];

#[must_use]
pub(super) fn collect_plot_tasks(
    layered: &LayeredRenderFrame,
    plan: &PartialCairoRenderPlan,
    viewport_width: f64,
) -> Vec<PanePartialRenderTask> {
    let mut tasks = Vec::new();

    for pane in &layered.panes {
        if !plan.targets_pane(pane.pane_id) {
            continue;
        }
        let pane_height = (pane.plot_bottom - pane.plot_top).max(0.0);
        if pane_height <= 0.0 {
            continue;
        }
        if let Some(frame) = layered.flatten_pane_layers(pane.pane_id, plan.plot_layers()) {
            if frame.is_empty() {
                continue;
            }
            tasks.push(PanePartialRenderTask::for_plot(
                frame,
                (0.0, pane.plot_top, viewport_width, pane_height),
            ));
        }
    }

    tasks
}

#[must_use]
pub(super) fn collect_main_axis_task(
    main_pane_id: PaneId,
    layered: &LayeredRenderFrame,
) -> Option<PanePartialRenderTask> {
    let frame = layered.flatten_pane_layers(main_pane_id, &AXIS_LAYERS)?;
    if frame.is_empty() {
        return None;
    }
    Some(PanePartialRenderTask::for_axis(frame))
}

#[cfg(test)]
mod tests {
    use super::{collect_main_axis_task, collect_plot_tasks};
    use crate::api::{
        InvalidationLevel, InvalidationMask, InvalidationTopic, InvalidationTopics,
        render_partial_test_support::layered_with_panes,
    };
    use crate::core::PaneId;
    use crate::render::{CanvasLayerKind, Color, TextHAlign, TextPrimitive};

    use super::super::render_partial_scheduler::PartialCairoRenderPlan;

    #[test]
    fn collect_plot_tasks_uses_plan_targets_and_clip() {
        let main = PaneId::new(0);
        let aux = PaneId::new(1);
        let mut layered = layered_with_panes(&[main, aux])
            .with_pane_regions(&[(main, 0.0, 200.0), (aux, 200.0, 500.0)]);

        layered.push_text(
            main,
            CanvasLayerKind::Crosshair,
            TextPrimitive::new(
                "main-cursor",
                30.0,
                40.0,
                11.0,
                Color::rgb(1.0, 1.0, 1.0),
                TextHAlign::Left,
            ),
        );
        layered.push_text(
            aux,
            CanvasLayerKind::Crosshair,
            TextPrimitive::new(
                "aux-cursor",
                30.0,
                260.0,
                11.0,
                Color::rgb(1.0, 1.0, 1.0),
                TextHAlign::Left,
            ),
        );

        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        );
        let plan = PartialCairoRenderPlan::build(pending, &[aux], &layered).expect("plan");

        let tasks = collect_plot_tasks(&layered, &plan, 800.0);
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].clear_region);
        assert_eq!(tasks[0].clip_rect, Some((0.0, 200.0, 800.0, 300.0)));
    }

    #[test]
    fn collect_main_axis_task_requires_non_empty_axis_frame() {
        let main = PaneId::new(0);
        let layered = layered_with_panes(&[main, PaneId::new(1)]);
        assert!(collect_main_axis_task(main, &layered).is_none());

        let mut layered = layered;
        layered.push_text(
            main,
            CanvasLayerKind::Axis,
            TextPrimitive::new(
                "axis",
                780.0,
                20.0,
                11.0,
                Color::rgb(1.0, 1.0, 1.0),
                TextHAlign::Right,
            ),
        );

        let task = collect_main_axis_task(main, &layered).expect("axis task");
        assert!(!task.clear_region);
        assert!(task.clip_rect.is_none());
    }
}
