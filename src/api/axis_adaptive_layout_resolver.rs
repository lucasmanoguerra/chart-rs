use crate::error::ChartResult;
use crate::render::Renderer;

use super::axis_layout_pass_resolver::resolve_axis_layout_pass;
use super::axis_price_axis_relayout_pass_resolver::resolve_price_axis_relayout_pass;
use super::axis_price_axis_relayout_resolver::should_relayout_price_axis_for_adaptive_width;
use super::axis_requested_section_sizes_resolver::resolve_requested_axis_section_sizes;
use super::layout_helpers::AxisLayout;
use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_adaptive_axis_layout(
        &self,
        style: RenderStyle,
        viewport_width: f64,
        viewport_height: f64,
        visible_start: f64,
        visible_end: f64,
    ) -> ChartResult<AxisLayout> {
        let requested_sections = resolve_requested_axis_section_sizes(style);
        let requested_price_axis_width = requested_sections.requested_price_axis_width;
        let requested_time_axis_height = requested_sections.requested_time_axis_height;

        let mut axis_layout = resolve_axis_layout_pass(
            viewport_width,
            viewport_height,
            requested_price_axis_width,
            requested_time_axis_height,
        );

        let adaptive_price_axis_width =
            self.resolve_adaptive_price_axis_width(style, axis_layout, visible_start, visible_end)?;

        if should_relayout_price_axis_for_adaptive_width(
            adaptive_price_axis_width,
            requested_price_axis_width,
        ) {
            axis_layout = resolve_price_axis_relayout_pass(
                viewport_width,
                viewport_height,
                requested_time_axis_height,
                adaptive_price_axis_width,
            );
        }

        Ok(axis_layout)
    }
}
