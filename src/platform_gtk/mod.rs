use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::{DrawingAreaExtManual, WidgetExt};

use crate::api::ChartEngine;
use crate::core::Viewport;
use crate::render::{CairoContextRenderer, Renderer};

/// Minimal GTK4 adapter that wires a `ChartEngine` into a `DrawingArea`.
///
/// The adapter keeps GTK concerns local while delegating all chart
/// state/transforms/render-command generation to the engine.
pub struct GtkChartAdapter<R: Renderer + CairoContextRenderer + 'static> {
    drawing_area: gtk::DrawingArea,
    engine: Rc<RefCell<ChartEngine<R>>>,
}

impl<R: Renderer + CairoContextRenderer + 'static> GtkChartAdapter<R> {
    #[must_use]
    pub fn new(engine: ChartEngine<R>) -> Self {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        let engine = Rc::new(RefCell::new(engine));
        let engine_for_draw = Rc::clone(&engine);
        drawing_area.set_draw_func(move |_widget, context, width, height| {
            if width <= 0 || height <= 0 {
                return;
            }

            let mut engine = match engine_for_draw.try_borrow_mut() {
                Ok(engine) => engine,
                Err(_) => return,
            };

            let viewport = Viewport::new(width as u32, height as u32);
            if engine.viewport() != viewport {
                let _ = engine.set_viewport(viewport);
            }

            let _ = engine.render_on_cairo_context(context);
        });

        Self {
            drawing_area,
            engine,
        }
    }

    #[must_use]
    pub fn drawing_area(&self) -> &gtk::DrawingArea {
        &self.drawing_area
    }

    #[must_use]
    pub fn engine(&self) -> Rc<RefCell<ChartEngine<R>>> {
        Rc::clone(&self.engine)
    }

    pub fn queue_draw(&self) {
        self.drawing_area.queue_draw();
    }
}
