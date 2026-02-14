use crate::core::PaneId;
use crate::error::ChartResult;
use crate::render::{LayeredRenderFrame, RenderFrame, Renderer};

use super::ChartEngine;
use super::axis_render_frame_builder::AxisRenderContext;
use super::crosshair_render_frame_builder::CrosshairRenderContext;
use super::series_scene_coordinator::SeriesSceneRenderContext;

impl<R: Renderer> ChartEngine<R> {
    /// Materializes backend-agnostic primitives for one draw pass.
    ///
    /// This keeps geometry computation deterministic and centralized in the API
    /// layer while renderer backends only execute drawing commands.
    pub fn build_render_frame(&self) -> ChartResult<RenderFrame> {
        self.build_render_outputs().map(|(frame, _)| frame)
    }

    /// Materializes a pane/layer aware render scene.
    ///
    /// This is the pane-oriented equivalent of `build_render_frame` and keeps
    /// canonical layer ordering explicit for parity work.
    pub fn build_layered_render_frame(&self) -> ChartResult<LayeredRenderFrame> {
        self.build_render_outputs().map(|(_, layered)| layered)
    }

    /// Materializes a pane-scoped frame for selective redraw paths.
    ///
    /// Returns `None` when `pane_id` is unknown.
    pub fn build_render_frame_for_pane(&self, pane_id: PaneId) -> ChartResult<Option<RenderFrame>> {
        let layered = self.build_layered_render_frame()?;
        Ok(layered.flatten_pane(pane_id))
    }

    fn build_render_outputs(&self) -> ChartResult<(RenderFrame, LayeredRenderFrame)> {
        let mut frame = RenderFrame::new(self.core.model.viewport);
        let main_pane_id = self.main_pane_id();
        let mut layered =
            LayeredRenderFrame::from_stacks(self.core.model.viewport, self.pane_layer_stacks());

        let (visible_start, visible_end) = self.core.model.time_scale.visible_range();

        let style = self.core.presentation.render_style;

        let resolved_layout = self.resolve_render_axis_layout(style, visible_start, visible_end)?;
        let viewport_width = resolved_layout.viewport_width;
        let viewport_height = resolved_layout.viewport_height;
        let visible_span_abs = resolved_layout.visible_span_abs;
        let plot_right = resolved_layout.axis_layout.plot_right;
        let plot_bottom = resolved_layout.axis_layout.plot_bottom;
        let pane_regions =
            self.resolve_pane_scene_regions(super::pane_scene_coordinator::PaneSceneContext {
                plot_top: 0.0,
                plot_bottom,
            });
        layered = self.apply_pane_scene_regions(layered, &pane_regions);
        self.append_series_scene_primitives(
            &mut frame,
            &mut layered,
            SeriesSceneRenderContext {
                main_pane_id,
                visible_start,
                visible_end,
                plot_right,
                style,
            },
        )?;
        let axis_display = self.append_axis_primitives(
            &mut frame,
            &mut layered,
            AxisRenderContext {
                main_pane_id,
                plot_right,
                plot_bottom,
                viewport_width,
                viewport_height,
                visible_start,
                visible_end,
                visible_span_abs,
                style,
            },
        )?;

        self.append_crosshair_primitives(
            &mut frame,
            &mut layered,
            CrosshairRenderContext {
                main_pane_id,
                plot_right,
                plot_bottom,
                viewport_width,
                viewport_height,
                visible_span_abs,
                fallback_display_base_price: axis_display.fallback_display_base_price,
                display_tick_step_abs: axis_display.display_tick_step_abs,
                display_suffix: axis_display.display_suffix,
                style,
            },
        )?;

        self.remap_plot_layers_into_pane_regions(&mut layered, &pane_regions, 0.0, plot_bottom);

        frame.validate()?;
        Ok((frame, layered))
    }
}
