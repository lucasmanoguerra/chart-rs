use cairo::{Context, Format, ImageSurface};
use pango::FontDescription;
use std::f64::consts::{FRAC_PI_2, PI};

use crate::error::{ChartError, ChartResult};
use crate::render::{Color, RenderFrame, Renderer, TextHAlign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CairoRenderStats {
    pub lines_drawn: usize,
    pub rects_drawn: usize,
    pub texts_drawn: usize,
}

/// Optional extension trait for renderers that can draw into an external Cairo
/// context (for example a GTK `DrawingArea` callback).
pub trait CairoContextRenderer {
    fn render_on_cairo_context(
        &mut self,
        context: &Context,
        frame: &RenderFrame,
    ) -> ChartResult<()>;
}

/// Cairo + Pango + PangoCairo renderer backend.
///
/// This renderer supports two modes:
/// - offscreen image-surface rendering through `Renderer::render`
/// - in-place rendering on an external Cairo context through
///   `CairoContextRenderer`
#[derive(Debug)]
pub struct CairoRenderer {
    surface: ImageSurface,
    clear_color: Color,
    last_stats: CairoRenderStats,
}

impl CairoRenderer {
    pub fn new(width: i32, height: i32) -> ChartResult<Self> {
        if width <= 0 || height <= 0 {
            return Err(ChartError::InvalidData(
                "cairo surface size must be > 0".to_owned(),
            ));
        }

        let surface = ImageSurface::create(Format::ARgb32, width, height)
            .map_err(|err| map_backend_error("failed to create cairo surface", err))?;
        Ok(Self {
            surface,
            clear_color: Color::rgb(1.0, 1.0, 1.0),
            last_stats: CairoRenderStats::default(),
        })
    }

    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        "cairo+pango+pangocairo"
    }

    #[must_use]
    pub fn surface(&self) -> &ImageSurface {
        &self.surface
    }

    #[must_use]
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, color: Color) -> ChartResult<()> {
        color.validate()?;
        self.clear_color = color;
        Ok(())
    }

    #[must_use]
    pub fn last_stats(&self) -> CairoRenderStats {
        self.last_stats
    }

    fn render_with_context(&mut self, context: &Context, frame: &RenderFrame) -> ChartResult<()> {
        frame.validate()?;
        self.clear_color.validate()?;

        apply_color(context, self.clear_color);
        context
            .paint()
            .map_err(|err| map_backend_error("failed to clear surface", err))?;

        let mut stats = CairoRenderStats::default();

        for line in &frame.lines {
            apply_color(context, line.color);
            context.set_line_width(line.stroke_width);
            context.move_to(line.x1, line.y1);
            context.line_to(line.x2, line.y2);
            context
                .stroke()
                .map_err(|err| map_backend_error("failed to stroke line", err))?;
            stats.lines_drawn += 1;
        }

        for rect in &frame.rects {
            append_rect_path(context, *rect);
            apply_color(context, rect.fill_color);
            if rect.border_width > 0.0 {
                context
                    .fill_preserve()
                    .map_err(|err| map_backend_error("failed to fill rectangle", err))?;
                apply_color(context, rect.border_color);
                context.set_line_width(rect.border_width);
                context
                    .stroke()
                    .map_err(|err| map_backend_error("failed to stroke rectangle border", err))?;
            } else {
                context
                    .fill()
                    .map_err(|err| map_backend_error("failed to fill rectangle", err))?;
            }
            stats.rects_drawn += 1;
        }

        for text in &frame.texts {
            let layout = pangocairo::functions::create_layout(context);
            let font_description =
                FontDescription::from_string(&format!("Sans {}", text.font_size_px));
            layout.set_font_description(Some(&font_description));
            layout.set_text(&text.text);

            let (text_width, _text_height) = layout.pixel_size();
            let x = match text.h_align {
                TextHAlign::Left => text.x,
                TextHAlign::Center => text.x - f64::from(text_width) / 2.0,
                TextHAlign::Right => text.x - f64::from(text_width),
            };

            apply_color(context, text.color);
            context.move_to(x, text.y);
            pangocairo::functions::show_layout(context, &layout);
            stats.texts_drawn += 1;
        }

        self.last_stats = stats;
        Ok(())
    }
}

impl Renderer for CairoRenderer {
    fn render(&mut self, frame: &RenderFrame) -> ChartResult<()> {
        let context = Context::new(&self.surface)
            .map_err(|err| map_backend_error("failed to create cairo context", err))?;
        self.render_with_context(&context, frame)
    }
}

impl CairoContextRenderer for CairoRenderer {
    fn render_on_cairo_context(
        &mut self,
        context: &Context,
        frame: &RenderFrame,
    ) -> ChartResult<()> {
        self.render_with_context(context, frame)
    }
}

fn apply_color(context: &Context, color: Color) {
    context.set_source_rgba(color.red, color.green, color.blue, color.alpha);
}

fn append_rect_path(context: &Context, rect: crate::render::RectPrimitive) {
    if rect.corner_radius <= 0.0 {
        context.rectangle(rect.x, rect.y, rect.width, rect.height);
        return;
    }

    let radius = rect
        .corner_radius
        .min(rect.width * 0.5)
        .min(rect.height * 0.5);
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;

    context.new_sub_path();
    context.arc(right - radius, top + radius, radius, -FRAC_PI_2, 0.0);
    context.arc(right - radius, bottom - radius, radius, 0.0, FRAC_PI_2);
    context.arc(left + radius, bottom - radius, radius, FRAC_PI_2, PI);
    context.arc(left + radius, top + radius, radius, PI, PI + FRAC_PI_2);
    context.close_path();
}

fn map_backend_error(prefix: &str, err: cairo::Error) -> ChartError {
    ChartError::InvalidData(format!("{prefix}: {err}"))
}
