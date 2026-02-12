use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, CrosshairGuideLineStyleBehavior};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn crosshair_guide_line_style_behavior(&self) -> CrosshairGuideLineStyleBehavior {
        let style = self.render_style();
        CrosshairGuideLineStyleBehavior {
            line_color: style.crosshair_line_color,
            line_width: style.crosshair_line_width,
            line_style: style.crosshair_line_style,
            horizontal_line_color: style.crosshair_horizontal_line_color,
            horizontal_line_width: style.crosshair_horizontal_line_width,
            horizontal_line_style: style.crosshair_horizontal_line_style,
            vertical_line_color: style.crosshair_vertical_line_color,
            vertical_line_width: style.crosshair_vertical_line_width,
            vertical_line_style: style.crosshair_vertical_line_style,
        }
    }

    pub fn set_crosshair_guide_line_style_behavior(
        &mut self,
        behavior: CrosshairGuideLineStyleBehavior,
    ) -> ChartResult<()> {
        let mut style = self.render_style();
        style.crosshair_line_color = behavior.line_color;
        style.crosshair_line_width = behavior.line_width;
        style.crosshair_line_style = behavior.line_style;
        style.crosshair_horizontal_line_color = behavior.horizontal_line_color;
        style.crosshair_horizontal_line_width = behavior.horizontal_line_width;
        style.crosshair_horizontal_line_style = behavior.horizontal_line_style;
        style.crosshair_vertical_line_color = behavior.vertical_line_color;
        style.crosshair_vertical_line_width = behavior.vertical_line_width;
        style.crosshair_vertical_line_style = behavior.vertical_line_style;
        self.set_render_style(style)
    }
}
