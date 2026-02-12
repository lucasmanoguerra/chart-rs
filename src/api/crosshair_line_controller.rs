use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, CrosshairGuideLineBehavior};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn crosshair_guide_line_behavior(&self) -> CrosshairGuideLineBehavior {
        let style = self.render_style();
        CrosshairGuideLineBehavior {
            show_lines: style.show_crosshair_lines,
            show_horizontal_line: style.show_crosshair_horizontal_line,
            show_vertical_line: style.show_crosshair_vertical_line,
        }
    }

    pub fn set_crosshair_guide_line_behavior(
        &mut self,
        behavior: CrosshairGuideLineBehavior,
    ) -> ChartResult<()> {
        let mut style = self.render_style();
        style.show_crosshair_lines = behavior.show_lines;
        style.show_crosshair_horizontal_line = behavior.show_horizontal_line;
        style.show_crosshair_vertical_line = behavior.show_vertical_line;
        self.set_render_style(style)
    }
}
