use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::{DrawingAreaExtManual, WidgetExt};

use crate::api::{
    ChartEngine, CrosshairFormatterDiagnostics, TimeCoordinateIndexPolicy, TimeFilledLogicalSlot,
};
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
    fn with_engine_ref<T>(
        &self,
        f: impl FnOnce(&ChartEngine<R>) -> ChartResult<T>,
        context: &str,
    ) -> ChartResult<T> {
        let engine = self.engine.try_borrow().map_err(|_| {
            crate::error::ChartError::InvalidData(format!(
                "failed to borrow chart engine for {context}"
            ))
        })?;
        f(&engine)
    }

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
        self.with_engine_ref(
            |engine| Ok(engine.crosshair_formatter_diagnostics()),
            "diagnostics",
        )
    }

    pub fn crosshair_formatter_diagnostics_json_contract_v1_pretty(&self) -> ChartResult<String> {
        self.with_engine_ref(
            ChartEngine::crosshair_formatter_diagnostics_json_contract_v1_pretty,
            "diagnostics export",
        )
    }

    pub fn snapshot_json_contract_v1_pretty(&self, body_width_px: f64) -> ChartResult<String> {
        self.with_engine_ref(
            |engine| engine.snapshot_json_contract_v1_pretty(body_width_px),
            "snapshot export",
        )
    }

    pub fn map_pixel_to_logical_index(
        &self,
        pixel: f64,
        policy: TimeCoordinateIndexPolicy,
    ) -> ChartResult<Option<f64>> {
        self.with_engine_ref(
            |engine| engine.map_pixel_to_logical_index(pixel, policy),
            "logical index mapping",
        )
    }

    pub fn map_pixel_to_logical_index_ceil(
        &self,
        pixel: f64,
        policy: TimeCoordinateIndexPolicy,
    ) -> ChartResult<Option<i64>> {
        self.with_engine_ref(
            |engine| engine.map_pixel_to_logical_index_ceil(pixel, policy),
            "logical index ceil mapping",
        )
    }

    pub fn map_logical_index_to_pixel(&self, logical_index: f64) -> ChartResult<Option<f64>> {
        self.with_engine_ref(
            |engine| engine.map_logical_index_to_pixel(logical_index),
            "logical to pixel mapping",
        )
    }

    pub fn nearest_filled_logical_slot_at_pixel(
        &self,
        pixel: f64,
    ) -> ChartResult<Option<TimeFilledLogicalSlot>> {
        self.with_engine_ref(
            |engine| engine.nearest_filled_logical_slot_at_pixel(pixel),
            "nearest filled logical slot lookup",
        )
    }

    pub fn next_filled_logical_index(&self, logical_index: f64) -> ChartResult<Option<f64>> {
        self.with_engine_ref(
            |engine| engine.next_filled_logical_index(logical_index),
            "next filled logical index lookup",
        )
    }

    pub fn prev_filled_logical_index(&self, logical_index: f64) -> ChartResult<Option<f64>> {
        self.with_engine_ref(
            |engine| engine.prev_filled_logical_index(logical_index),
            "previous filled logical index lookup",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{GtkChartAdapter, gtk};
    use crate::api::{ChartEngine, ChartEngineConfig, TimeCoordinateIndexPolicy};
    use crate::core::{DataPoint, Viewport};
    use crate::render::CairoRenderer;

    fn build_adapter() -> Option<GtkChartAdapter<CairoRenderer>> {
        if gtk::is_initialized() && !gtk::is_initialized_main_thread() {
            return None;
        }
        if !gtk::is_initialized_main_thread() && gtk::init().is_err() {
            return None;
        }

        let renderer = CairoRenderer::new(16, 16).expect("renderer");
        let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
            .with_price_domain(0.0, 1.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine");
        engine.set_data(vec![
            DataPoint::new(0.0, 10.0),
            DataPoint::new(10.0, 11.0),
            DataPoint::new(20.0, 12.0),
            DataPoint::new(50.0, 15.0),
        ]);
        Some(GtkChartAdapter::new(engine))
    }

    #[test]
    fn gtk_adapter_exposes_filled_slot_navigation_and_coordinate_policy_utilities() {
        let Some(adapter) = build_adapter() else {
            return;
        };

        let x = adapter
            .map_logical_index_to_pixel(2.0)
            .expect("logical->pixel")
            .expect("space");
        let nearest = adapter
            .nearest_filled_logical_slot_at_pixel(x)
            .expect("nearest")
            .expect("slot");
        assert!((nearest.logical_index - 2.0).abs() <= 1e-9);
        assert!((nearest.time - 20.0).abs() <= 1e-9);

        let next = adapter
            .next_filled_logical_index(2.1)
            .expect("next logical index");
        let prev = adapter
            .prev_filled_logical_index(4.9)
            .expect("previous logical index");
        assert_eq!(next, Some(5.0));
        assert_eq!(prev, Some(2.0));

        let x_policy = adapter
            .map_logical_index_to_pixel(2.2)
            .expect("logical->pixel")
            .expect("space");
        let allow = adapter
            .map_pixel_to_logical_index_ceil(x_policy, TimeCoordinateIndexPolicy::AllowWhitespace)
            .expect("allow whitespace")
            .expect("index");
        let ignore = adapter
            .map_pixel_to_logical_index_ceil(x_policy, TimeCoordinateIndexPolicy::IgnoreWhitespace)
            .expect("ignore whitespace")
            .expect("index");
        assert_eq!(allow, 3);
        assert_eq!(ignore, 2);
    }
}
