use crate::error::{ChartError, ChartResult};

/// RGBA color in normalized 0..=1 channel values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}

impl Color {
    #[must_use]
    pub const fn rgba(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    #[must_use]
    pub const fn rgb(red: f64, green: f64, blue: f64) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    pub fn validate(self) -> ChartResult<()> {
        for (channel, value) in [
            ("red", self.red),
            ("green", self.green),
            ("blue", self.blue),
            ("alpha", self.alpha),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(ChartError::InvalidData(format!(
                    "color channel `{channel}` must be finite and in [0, 1]"
                )));
            }
        }
        Ok(())
    }
}

/// Draw command for one line segment in pixel space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinePrimitive {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub stroke_width: f64,
    pub color: Color,
}

impl LinePrimitive {
    #[must_use]
    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64, stroke_width: f64, color: Color) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            stroke_width,
            color,
        }
    }

    pub fn validate(self) -> ChartResult<()> {
        if !self.x1.is_finite()
            || !self.y1.is_finite()
            || !self.x2.is_finite()
            || !self.y2.is_finite()
        {
            return Err(ChartError::InvalidData(
                "line coordinates must be finite".to_owned(),
            ));
        }
        if !self.stroke_width.is_finite() || self.stroke_width <= 0.0 {
            return Err(ChartError::InvalidData(
                "line stroke width must be finite and > 0".to_owned(),
            ));
        }
        self.color.validate()
    }
}

/// Draw command for one filled rectangle in pixel space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPrimitive {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub fill_color: Color,
    pub border_width: f64,
    pub border_color: Color,
    pub corner_radius: f64,
}

impl RectPrimitive {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64, fill_color: Color) -> Self {
        Self {
            x,
            y,
            width,
            height,
            fill_color,
            border_width: 0.0,
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
            corner_radius: 0.0,
        }
    }

    #[must_use]
    pub fn with_border(mut self, border_width: f64, border_color: Color) -> Self {
        self.border_width = border_width;
        self.border_color = border_color;
        self
    }

    #[must_use]
    pub fn with_corner_radius(mut self, corner_radius: f64) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn validate(self) -> ChartResult<()> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(ChartError::InvalidData(
                "rect coordinates must be finite".to_owned(),
            ));
        }
        if !self.width.is_finite()
            || !self.height.is_finite()
            || self.width <= 0.0
            || self.height <= 0.0
        {
            return Err(ChartError::InvalidData(
                "rect size must be finite and > 0".to_owned(),
            ));
        }
        if !self.border_width.is_finite() || self.border_width < 0.0 {
            return Err(ChartError::InvalidData(
                "rect border width must be finite and >= 0".to_owned(),
            ));
        }
        if !self.corner_radius.is_finite() || self.corner_radius < 0.0 {
            return Err(ChartError::InvalidData(
                "rect corner radius must be finite and >= 0".to_owned(),
            ));
        }
        if self.corner_radius > self.width * 0.5 || self.corner_radius > self.height * 0.5 {
            return Err(ChartError::InvalidData(
                "rect corner radius must be <= half of rect size".to_owned(),
            ));
        }
        self.fill_color.validate()?;
        if self.border_width > 0.0 {
            self.border_color.validate()?;
        }
        Ok(())
    }
}

/// Horizontal text alignment relative to `TextPrimitive::x`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextHAlign {
    Left,
    Center,
    Right,
}

/// Draw command for one label in pixel space.
#[derive(Debug, Clone, PartialEq)]
pub struct TextPrimitive {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size_px: f64,
    pub color: Color,
    pub h_align: TextHAlign,
}

impl TextPrimitive {
    #[must_use]
    pub fn new(
        text: impl Into<String>,
        x: f64,
        y: f64,
        font_size_px: f64,
        color: Color,
        h_align: TextHAlign,
    ) -> Self {
        Self {
            text: text.into(),
            x,
            y,
            font_size_px,
            color,
            h_align,
        }
    }

    pub fn validate(&self) -> ChartResult<()> {
        if self.text.is_empty() {
            return Err(ChartError::InvalidData(
                "text primitive must not be empty".to_owned(),
            ));
        }
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(ChartError::InvalidData(
                "text coordinates must be finite".to_owned(),
            ));
        }
        if !self.font_size_px.is_finite() || self.font_size_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "font size must be finite and > 0".to_owned(),
            ));
        }
        self.color.validate()
    }
}
