use crate::core::Viewport;
use crate::error::{ChartError, ChartResult};
use crate::render::{LinePrimitive, TextPrimitive};

/// Backend-agnostic scene for one chart draw pass.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrame {
    pub viewport: Viewport,
    pub lines: Vec<LinePrimitive>,
    pub texts: Vec<TextPrimitive>,
}

impl RenderFrame {
    #[must_use]
    pub fn new(viewport: Viewport) -> Self {
        Self {
            viewport,
            lines: Vec::new(),
            texts: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_line(mut self, line: LinePrimitive) -> Self {
        self.lines.push(line);
        self
    }

    #[must_use]
    pub fn with_text(mut self, text: TextPrimitive) -> Self {
        self.texts.push(text);
        self
    }

    pub fn validate(&self) -> ChartResult<()> {
        if !self.viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: self.viewport.width,
                height: self.viewport.height,
            });
        }

        for line in &self.lines {
            line.validate()?;
        }
        for text in &self.texts {
            text.validate()?;
        }

        Ok(())
    }
}
