#[path = "shared/mod.rs"]
mod shared;

use std::cell::Cell;
use std::rc::Rc;

use chart_rs::api::{ChartEngine, ChartEngineConfig, RenderStyle};
use chart_rs::core::{
    AreaGeometry, BarGeometry, BaselineGeometry, CandleGeometry, HistogramBar, LineSegment,
    TimeScaleTuning, Viewport,
};
use chart_rs::extensions::{MarkerPlacementConfig, MarkerPosition, MarkerSide, SeriesMarker};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{CairoRenderer, Color};
use gtk4 as gtk;
use gtk4::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayMode {
    Line,
    Area,
    Baseline,
    Histogram,
    Bars,
    Candles,
    Markers,
}

impl OverlayMode {
    const ALL: [Self; 7] = [
        Self::Line,
        Self::Area,
        Self::Baseline,
        Self::Histogram,
        Self::Bars,
        Self::Candles,
        Self::Markers,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Area => "area",
            Self::Baseline => "baseline",
            Self::Histogram => "histogram",
            Self::Bars => "bars",
            Self::Candles => "candles",
            Self::Markers => "markers",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Line => "Line Segments",
            Self::Area => "Area Geometry",
            Self::Baseline => "Baseline Geometry",
            Self::Histogram => "Histogram Bars",
            Self::Bars => "OHLC Bars",
            Self::Candles => "Candlestick Bodies",
            Self::Markers => "Markers On Candles",
        }
    }

    fn from_id(id: &str) -> Self {
        match id {
            "line" => Self::Line,
            "area" => Self::Area,
            "baseline" => Self::Baseline,
            "histogram" => Self::Histogram,
            "bars" => Self::Bars,
            "candles" => Self::Candles,
            "markers" => Self::Markers,
            _ => Self::Candles,
        }
    }
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_projection_gallery")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let points = shared::build_wave_points(680, 0.0, 1.0, 130.0);
    let candles = match shared::build_candles_from_points(&points) {
        Ok(candles) => candles,
        Err(err) => {
            eprintln!("failed to build candle dataset: {err}");
            return;
        }
    };

    let markers = Rc::new(build_markers(&candles));

    let renderer = match CairoRenderer::new(1280, 800) {
        Ok(renderer) => renderer,
        Err(err) => {
            eprintln!("failed to initialize cairo renderer: {err}");
            return;
        }
    };

    let config = ChartEngineConfig::new(Viewport::new(1280, 800), 0.0, 680.0)
        .with_price_domain(90.0, 220.0)
        .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);
    let mut engine = match ChartEngine::new(renderer, config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("failed to initialize projection gallery engine: {err}");
            return;
        }
    };

    engine.set_data(points);
    engine.set_candles(candles);
    if let Err(err) = engine.fit_time_to_data(TimeScaleTuning::default()) {
        eprintln!("fit_time_to_data failed: {err}");
    }
    if let Err(err) = engine.autoscale_price_from_candles() {
        eprintln!("autoscale_price_from_candles failed: {err}");
    }

    let style = RenderStyle {
        series_line_color: Color::rgba(0.10, 0.10, 0.10, 0.15),
        show_crosshair_time_label_box: true,
        show_crosshair_price_label_box: true,
        ..engine.render_style()
    };
    if let Err(err) = engine.set_render_style(style) {
        eprintln!("set_render_style failed: {err}");
    }

    let engine = Rc::new(std::cell::RefCell::new(engine));

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    let mode = Rc::new(Cell::new(OverlayMode::Candles));
    let overscan = Rc::new(Cell::new(false));
    let baseline = Rc::new(Cell::new(130.0));

    drawing_area.set_draw_func({
        let engine = Rc::clone(&engine);
        let mode = Rc::clone(&mode);
        let overscan = Rc::clone(&overscan);
        let baseline = Rc::clone(&baseline);
        let markers = Rc::clone(&markers);

        move |_widget, context, width, height| {
            if width <= 0 || height <= 0 {
                return;
            }

            let mut chart = match engine.try_borrow_mut() {
                Ok(chart) => chart,
                Err(_) => return,
            };

            let viewport = Viewport::new(width as u32, height as u32);
            if chart.viewport() != viewport {
                let _ = chart.set_viewport(viewport);
            }

            let _ = chart.render_on_cairo_context(context);

            match mode.get() {
                OverlayMode::Line => {
                    if let Ok(segments) = chart.project_line_segments() {
                        draw_line_segments(context, &segments);
                    }
                }
                OverlayMode::Area => {
                    let geometry = if overscan.get() {
                        chart.project_visible_area_geometry_with_overscan(0.08)
                    } else {
                        chart.project_visible_area_geometry()
                    };
                    if let Ok(geometry) = geometry {
                        draw_area(context, &geometry);
                    }
                }
                OverlayMode::Baseline => {
                    let baseline_price = baseline.get();
                    let geometry = if overscan.get() {
                        chart.project_visible_baseline_geometry_with_overscan(baseline_price, 0.08)
                    } else {
                        chart.project_visible_baseline_geometry(baseline_price)
                    };
                    if let Ok(geometry) = geometry {
                        draw_baseline(context, &geometry);
                    }
                }
                OverlayMode::Histogram => {
                    let baseline_price = baseline.get();
                    let bars = if overscan.get() {
                        chart.project_visible_histogram_bars_with_overscan(
                            9.0,
                            baseline_price,
                            0.08,
                        )
                    } else {
                        chart.project_visible_histogram_bars(9.0, baseline_price)
                    };
                    if let Ok(bars) = bars {
                        draw_histogram(context, &bars);
                    }
                }
                OverlayMode::Bars => {
                    let bars = if overscan.get() {
                        chart.project_visible_bars_with_overscan(8.0, 0.08)
                    } else {
                        chart.project_visible_bars(8.0)
                    };
                    if let Ok(bars) = bars {
                        draw_ohlc_bars(context, &bars);
                    }
                }
                OverlayMode::Candles => {
                    let candles = if overscan.get() {
                        chart.project_visible_candles_with_overscan(8.0, 0.08)
                    } else {
                        chart.project_visible_candles(8.0)
                    };
                    if let Ok(candles) = candles {
                        draw_candles(context, &candles);
                    }
                }
                OverlayMode::Markers => {
                    if let Ok(candles) = chart.project_visible_candles(7.0) {
                        draw_candles(context, &candles);
                    }
                    let placed = if overscan.get() {
                        chart.project_visible_markers_on_candles_with_overscan(
                            markers.as_slice(),
                            0.08,
                            MarkerPlacementConfig::default(),
                        )
                    } else {
                        chart.project_visible_markers_on_candles(
                            markers.as_slice(),
                            MarkerPlacementConfig::default(),
                        )
                    };
                    if let Ok(placed) = placed {
                        draw_markers(context, &placed);
                    }
                }
            }
        }
    });

    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let mode_combo = gtk::ComboBoxText::new();
    for mode_entry in OverlayMode::ALL {
        mode_combo.append(Some(mode_entry.id()), mode_entry.label());
    }
    mode_combo.set_active_id(Some(OverlayMode::Candles.id()));

    let overscan_toggle = gtk::CheckButton::with_label("Visible + overscan");
    overscan_toggle.set_active(false);

    let crosshair_normal_toggle = gtk::CheckButton::with_label("Normal crosshair (off=Magnet)");
    crosshair_normal_toggle.set_active(true);

    let baseline_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 80.0, 220.0, 0.5);
    baseline_scale.set_value(130.0);
    baseline_scale.set_hexpand(true);

    let fit_button = gtk::Button::with_label("Fit + Autoscale");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&gtk::Label::new(Some("Mode:")));
    controls.append(&mode_combo);
    controls.append(&overscan_toggle);
    controls.append(&crosshair_normal_toggle);
    controls.append(&gtk::Label::new(Some("Baseline:")));
    controls.append(&baseline_scale);
    controls.append(&fit_button);

    {
        let mode = Rc::clone(&mode);
        let drawing_area = drawing_area.clone();
        mode_combo.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                mode.set(OverlayMode::from_id(id.as_str()));
                drawing_area.queue_draw();
            }
        });
    }

    {
        let overscan = Rc::clone(&overscan);
        let drawing_area = drawing_area.clone();
        overscan_toggle.connect_toggled(move |toggle| {
            overscan.set(toggle.is_active());
            drawing_area.queue_draw();
        });
    }

    {
        let baseline = Rc::clone(&baseline);
        let drawing_area = drawing_area.clone();
        baseline_scale.connect_value_changed(move |scale| {
            baseline.set(scale.value());
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        crosshair_normal_toggle.connect_toggled(move |toggle| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                if toggle.is_active() {
                    chart.set_crosshair_mode(CrosshairMode::Normal);
                } else {
                    chart.set_crosshair_mode(CrosshairMode::Magnet);
                }
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        fit_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.fit_time_to_data(TimeScaleTuning::default());
                let _ = chart.autoscale_price_from_candles();
            }
            drawing_area.queue_draw();
        });
    }

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);
    {
        let engine = Rc::clone(&engine);
        let mode = Rc::clone(&mode);
        let overscan = Rc::clone(&overscan);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(140), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();
                status_label.set_text(&format!(
                    "mode={} overscan={} t=[{time_start:.2}, {time_end:.2}] p=[{price_min:.2}, {price_max:.2}]",
                    mode.get().label(),
                    overscan.get(),
                ));
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let instructions = gtk::Label::new(Some(
        "Projection gallery: cambie el modo para inspeccionar la geometria proyectada por API (project_*). Mouse: mover/rueda/drag.",
    ));
    instructions.set_xalign(0.0);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(10);
    root.set_margin_bottom(10);
    root.set_margin_start(10);
    root.set_margin_end(10);
    root.append(&instructions);
    root.append(&controls);
    root.append(&status_label);
    root.append(&drawing_area);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("chart-rs GTK Projection Gallery")
        .default_width(1280)
        .default_height(820)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn build_markers(candles: &[chart_rs::core::OhlcBar]) -> Vec<SeriesMarker> {
    candles
        .iter()
        .enumerate()
        .step_by(18)
        .map(|(index, candle)| {
            let position = if index % 3 == 0 {
                MarkerPosition::AboveBar
            } else if index % 3 == 1 {
                MarkerPosition::BelowBar
            } else {
                MarkerPosition::InBar
            };
            SeriesMarker::new(format!("m-{index}"), candle.time, position)
                .with_text(format!("M{index}"))
                .with_priority((index % 5) as i32)
        })
        .collect()
}

fn draw_line_segments(context: &gtk::cairo::Context, segments: &[LineSegment]) {
    context.set_source_rgba(0.09, 0.56, 0.82, 0.95);
    context.set_line_width(2.2);
    for segment in segments {
        context.move_to(segment.x1, segment.y1);
        context.line_to(segment.x2, segment.y2);
    }
    let _ = context.stroke();
}

fn draw_area(context: &gtk::cairo::Context, geometry: &AreaGeometry) {
    if geometry.fill_polygon.is_empty() {
        return;
    }

    context.set_source_rgba(0.06, 0.68, 0.39, 0.20);
    if let Some(first) = geometry.fill_polygon.first() {
        context.move_to(first.x, first.y);
        for vertex in &geometry.fill_polygon[1..] {
            context.line_to(vertex.x, vertex.y);
        }
        context.close_path();
        let _ = context.fill();
    }

    context.set_source_rgba(0.04, 0.46, 0.27, 0.95);
    context.set_line_width(2.0);
    if let Some(first) = geometry.line_points.first() {
        context.move_to(first.x, first.y);
        for vertex in &geometry.line_points[1..] {
            context.line_to(vertex.x, vertex.y);
        }
        let _ = context.stroke();
    }
}

fn draw_baseline(context: &gtk::cairo::Context, geometry: &BaselineGeometry) {
    if geometry.line_points.is_empty() {
        return;
    }

    if let Some(first) = geometry.above_fill_polygon.first() {
        context.set_source_rgba(0.00, 0.70, 0.40, 0.20);
        context.move_to(first.x, first.y);
        for vertex in &geometry.above_fill_polygon[1..] {
            context.line_to(vertex.x, vertex.y);
        }
        context.close_path();
        let _ = context.fill();
    }

    if let Some(first) = geometry.below_fill_polygon.first() {
        context.set_source_rgba(0.90, 0.25, 0.22, 0.18);
        context.move_to(first.x, first.y);
        for vertex in &geometry.below_fill_polygon[1..] {
            context.line_to(vertex.x, vertex.y);
        }
        context.close_path();
        let _ = context.fill();
    }

    context.set_source_rgba(0.95, 0.62, 0.08, 0.92);
    context.set_line_width(1.2);
    context.move_to(0.0, geometry.baseline_y);
    context.line_to(20_000.0, geometry.baseline_y);
    let _ = context.stroke();

    context.set_source_rgba(0.10, 0.10, 0.10, 0.9);
    context.set_line_width(1.8);
    if let Some(first) = geometry.line_points.first() {
        context.move_to(first.x, first.y);
        for vertex in &geometry.line_points[1..] {
            context.line_to(vertex.x, vertex.y);
        }
        let _ = context.stroke();
    }
}

fn draw_histogram(context: &gtk::cairo::Context, bars: &[HistogramBar]) {
    context.set_source_rgba(0.28, 0.55, 0.96, 0.65);
    for bar in bars {
        context.rectangle(
            bar.x_left,
            bar.y_top,
            (bar.x_right - bar.x_left).max(1.0),
            (bar.y_bottom - bar.y_top).max(1.0),
        );
    }
    let _ = context.fill();
}

fn draw_ohlc_bars(context: &gtk::cairo::Context, bars: &[BarGeometry]) {
    context.set_line_width(1.4);
    for bar in bars {
        context.set_source_rgba(0.12, 0.12, 0.12, 0.95);
        context.move_to(bar.center_x, bar.high_y);
        context.line_to(bar.center_x, bar.low_y);

        context.move_to(bar.open_x, bar.open_y);
        context.line_to(bar.center_x, bar.open_y);

        context.move_to(bar.center_x, bar.close_y);
        context.line_to(bar.close_x, bar.close_y);
    }
    let _ = context.stroke();
}

fn draw_candles(context: &gtk::cairo::Context, candles: &[CandleGeometry]) {
    context.set_line_width(1.1);

    for candle in candles {
        let (r, g, b) = if candle.is_bullish {
            (0.00, 0.66, 0.35)
        } else {
            (0.85, 0.20, 0.20)
        };

        context.set_source_rgba(r, g, b, 0.95);
        context.move_to(candle.center_x, candle.wick_top);
        context.line_to(candle.center_x, candle.wick_bottom);
        let _ = context.stroke();

        context.set_source_rgba(r, g, b, 0.35);
        context.rectangle(
            candle.body_left,
            candle.body_top,
            (candle.body_right - candle.body_left).max(1.0),
            (candle.body_bottom - candle.body_top).max(1.0),
        );
        let _ = context.fill_preserve();

        context.set_source_rgba(r, g, b, 0.95);
        context.set_line_width(1.0);
        let _ = context.stroke();
    }
}

fn draw_markers(context: &gtk::cairo::Context, markers: &[chart_rs::extensions::PlacedMarker]) {
    context.set_font_size(11.0);

    for marker in markers {
        let (r, g, b) = match marker.side {
            MarkerSide::Above => (0.18, 0.50, 0.95),
            MarkerSide::Below => (0.95, 0.36, 0.24),
            MarkerSide::Center => (0.08, 0.66, 0.36),
        };

        context.set_source_rgba(r, g, b, 0.95);
        context.arc(marker.x, marker.y, 4.4, 0.0, std::f64::consts::TAU);
        let _ = context.fill();

        if let Some(label) = &marker.label {
            context.set_source_rgba(0.08, 0.08, 0.08, 0.85);
            context.rectangle(label.left_px, label.top_px, label.width_px, label.height_px);
            let _ = context.fill();

            context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            context.move_to(label.left_px + 5.0, label.top_px + label.height_px - 4.0);
            let _ = context.show_text(&label.text);
        }
    }
}
