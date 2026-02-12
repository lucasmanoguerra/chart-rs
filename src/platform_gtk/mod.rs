use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::{DrawingAreaExtManual, WidgetExt};

use crate::api::{ChartEngine, CrosshairFormatterDiagnostics};
use crate::core::Viewport;
use crate::error::ChartResult;
use crate::render::{CairoContextRenderer, Renderer};

/// Minimal GTK4 adapter that wires a `ChartEngine` into a `DrawingArea`.
///
/// The adapter keeps GTK concerns local while delegating all chart
/// state/transforms/render-command generation to the engine.
type CrosshairDiagnosticsHook = Rc<dyn Fn(CrosshairFormatterDiagnostics)>;
type SnapshotJsonHook = Rc<dyn Fn(String)>;

pub struct GtkChartAdapter<R: Renderer + CairoContextRenderer + 'static> {
    drawing_area: gtk::DrawingArea,
    engine: Rc<RefCell<ChartEngine<R>>>,
    diagnostics_hook: Rc<RefCell<Option<CrosshairDiagnosticsHook>>>,
    snapshot_hook: Rc<RefCell<Option<SnapshotJsonHook>>>,
    snapshot_hook_body_width_px: Rc<RefCell<f64>>,
}

impl<R: Renderer + CairoContextRenderer + 'static> GtkChartAdapter<R> {
    #[must_use]
    pub fn new(engine: ChartEngine<R>) -> Self {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        let engine = Rc::new(RefCell::new(engine));
        let diagnostics_hook: Rc<RefCell<Option<CrosshairDiagnosticsHook>>> =
            Rc::new(RefCell::new(None));
        let snapshot_hook: Rc<RefCell<Option<SnapshotJsonHook>>> = Rc::new(RefCell::new(None));
        let snapshot_hook_body_width_px = Rc::new(RefCell::new(7.0));
        let engine_for_draw = Rc::clone(&engine);
        let diagnostics_hook_for_draw = Rc::clone(&diagnostics_hook);
        let snapshot_hook_for_draw = Rc::clone(&snapshot_hook);
        let snapshot_hook_body_width_px_for_draw = Rc::clone(&snapshot_hook_body_width_px);
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

            if let Some(hook) = diagnostics_hook_for_draw.borrow().as_ref().cloned() {
                hook(engine.crosshair_formatter_diagnostics());
            }

            if let Some(hook) = snapshot_hook_for_draw.borrow().as_ref().cloned() {
                let body_width_px = *snapshot_hook_body_width_px_for_draw.borrow();
                if let Ok(json) = engine.snapshot_json_contract_v1_pretty(body_width_px) {
                    hook(json);
                }
            }
        });

        Self {
            drawing_area,
            engine,
            diagnostics_hook,
            snapshot_hook,
            snapshot_hook_body_width_px,
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

    pub fn set_crosshair_diagnostics_hook<F>(&self, hook: F)
    where
        F: Fn(CrosshairFormatterDiagnostics) + 'static,
    {
        *self.diagnostics_hook.borrow_mut() = Some(Rc::new(hook));
    }

    pub fn clear_crosshair_diagnostics_hook(&self) {
        *self.diagnostics_hook.borrow_mut() = None;
    }

    pub fn set_snapshot_json_hook<F>(&self, body_width_px: f64, hook: F)
    where
        F: Fn(String) + 'static,
    {
        *self.snapshot_hook_body_width_px.borrow_mut() = body_width_px;
        *self.snapshot_hook.borrow_mut() = Some(Rc::new(hook));
    }

    pub fn clear_snapshot_json_hook(&self) {
        *self.snapshot_hook.borrow_mut() = None;
    }

    pub fn crosshair_formatter_diagnostics(&self) -> ChartResult<CrosshairFormatterDiagnostics> {
        let engine = self.engine.try_borrow().map_err(|_| {
            crate::error::ChartError::InvalidData(
                "failed to borrow chart engine for diagnostics".to_owned(),
            )
        })?;
        Ok(engine.crosshair_formatter_diagnostics())
    }

    pub fn crosshair_formatter_diagnostics_json_contract_v1_pretty(&self) -> ChartResult<String> {
        let engine = self.engine.try_borrow().map_err(|_| {
            crate::error::ChartError::InvalidData(
                "failed to borrow chart engine for diagnostics export".to_owned(),
            )
        })?;
        engine.crosshair_formatter_diagnostics_json_contract_v1_pretty()
    }

    pub fn snapshot_json_contract_v1_pretty(&self, body_width_px: f64) -> ChartResult<String> {
        let engine = self.engine.try_borrow().map_err(|_| {
            crate::error::ChartError::InvalidData(
                "failed to borrow chart engine for snapshot export".to_owned(),
            )
        })?;
        engine.snapshot_json_contract_v1_pretty(body_width_px)
    }
}
