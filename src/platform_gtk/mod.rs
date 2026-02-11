use gtk4 as gtk;

use crate::api::ChartEngine;
use crate::render::Renderer;

pub struct GtkChartAdapter<R: Renderer> {
    _engine: ChartEngine<R>,
}

impl<R: Renderer> GtkChartAdapter<R> {
    #[must_use]
    pub fn new(engine: ChartEngine<R>) -> Self {
        let _ = std::mem::size_of::<gtk::DrawingArea>();
        Self { _engine: engine }
    }
}
