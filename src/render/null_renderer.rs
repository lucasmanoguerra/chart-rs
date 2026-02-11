use crate::error::ChartResult;
use crate::render::{RenderFrame, Renderer};

/// No-op renderer used by tests and headless engine usage.
///
/// It still validates frame content so tests can catch invalid geometry before
/// a real backend is introduced.
#[derive(Debug, Default)]
pub struct NullRenderer {
    pub last_line_count: usize,
    pub last_text_count: usize,
}

impl Renderer for NullRenderer {
    fn render(&mut self, frame: &RenderFrame) -> ChartResult<()> {
        frame.validate()?;
        self.last_line_count = frame.lines.len();
        self.last_text_count = frame.texts.len();
        Ok(())
    }
}
