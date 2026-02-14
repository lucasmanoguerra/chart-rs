use crate::core::Viewport;
use crate::error::{ChartError, ChartResult};
use crate::render::{LinePrimitive, RectPrimitive, TextPrimitive};

/// Backend-agnostic scene for one chart draw pass.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrame {
    pub viewport: Viewport,
    pub lines: Vec<LinePrimitive>,
    pub rects: Vec<RectPrimitive>,
    pub texts: Vec<TextPrimitive>,
}

impl RenderFrame {
    #[must_use]
    pub fn new(viewport: Viewport) -> Self {
        Self {
            viewport,
            lines: Vec::new(),
            rects: Vec::new(),
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

    #[must_use]
    pub fn with_rect(mut self, rect: RectPrimitive) -> Self {
        self.rects.push(rect);
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
        for rect in &self.rects {
            rect.validate()?;
        }
        for text in &self.texts {
            text.validate()?;
        }

        Ok(())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.rects.is_empty() && self.texts.is_empty()
    }
}
