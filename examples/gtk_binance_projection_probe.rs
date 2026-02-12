#[path = "shared/mod.rs"]
mod shared;

use std::cell::Cell;
use std::rc::Rc;

use chart_rs::api::ChartEngineConfig;
use chart_rs::core::{
    AreaGeometry, BarGeometry, BaselineGeometry, CandleGeometry, HistogramBar, LineSegment,
    TimeScaleTuning, Viewport,
};
use chart_rs::extensions::{MarkerPlacementConfig, MarkerPosition, MarkerSide, SeriesMarker};
use chart_rs::render::{CairoRenderer, Color, RenderFrame};
use gtk4 as gtk;
use gtk4::prelude::*;

use shared::binance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionMode {
    Candles,
    Bars,
    Line,
    Area,
    Baseline,
    Histogram,
    Markers,
}

impl ProjectionMode {
    const ALL: [Self; 7] = [
        Self::Candles,
        Self::Bars,
        Self::Line,
        Self::Area,
        Self::Baseline,
        Self::Histogram,
        Self::Markers,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::Candles => "candles",
            Self::Bars => "bars",
            Self::Line => "line",
            Self::Area => "area",
            Self::Baseline => "baseline",
            Self::Histogram => "hist",
            Self::Markers => "markers",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Candles => "Candles",
            Self::Bars => "OHLC Bars",
            Self::Line => "Line Segments",
            Self::Area => "Area Geometry",
            Self::Baseline => "Baseline Geometry",
            Self::Histogram => "Histogram",
            Self::Markers => "Markers",
        }
    }

    fn from_id(id: &str) -> Self {
        match id {
            "bars" => Self::Bars,
            "line" => Self::Line,
            "area" => Self::Area,
            "baseline" => Self::Baseline,
            "hist" => Self::Histogram,
            "markers" => Self::Markers,
            _ => Self::Candles,
        }
    }
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_binance_projection_probe")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let data = match binance::fetch_market_data("BTCUSDT", "1h", 420) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("binance fetch failed, using fallback dataset: {err}");
            binance::fallback_market_data("BTCUSDT", "1h")
        }
    };

    let (time_start, time_end) = binance::data_time_range(&data);
    let (price_min, price_max) = binance::data_price_range(&data);

    let renderer = match CairoRenderer::new(1320, 860) {
        Ok(renderer) => renderer,
        Err(err) => {
            eprintln!("renderer error: {err}");
            return;
        }
    };

    let config = ChartEngineConfig::new(Viewport::new(1320, 860), time_start, time_end)
        .with_price_domain(price_min, price_max)
        .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);
    let mut engine = match chart_rs::api::ChartEngine::new(renderer, config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("engine init error: {err}");
            return;
        }
    };
    engine.set_data(data.close_points.clone());
    engine.set_candles(data.candles.clone());
    let _ = engine.fit_time_to_data(TimeScaleTuning::default());
    let _ = engine.autoscale_price_from_candles();

    let engine = Rc::new(std::cell::RefCell::new(engine));
    let markers = Rc::new(std::cell::RefCell::new(build_markers(&data.candles)));
    let mode = Rc::new(Cell::new(ProjectionMode::Candles));
    let overscan = Rc::new(Cell::new(true));
    let baseline = Rc::new(Cell::new((price_min + price_max) * 0.5));

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    drawing_area.set_draw_func({
        let engine = Rc::clone(&engine);
        let markers = Rc::clone(&markers);
        let mode = Rc::clone(&mode);
        let overscan = Rc::clone(&overscan);
        let baseline = Rc::clone(&baseline);
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
                ProjectionMode::Candles => {
                    let out = if overscan.get() {
                        chart.project_visible_candles_with_overscan(8.0, 0.06)
                    } else {
                        chart.project_visible_candles(8.0)
                    };
                    if let Ok(candles) = out {
                        draw_candles(context, &candles);
                    }
                }
                ProjectionMode::Bars => {
                    let out = if overscan.get() {
                        chart.project_visible_bars_with_overscan(8.0, 0.06)
                    } else {
                        chart.project_visible_bars(8.0)
                    };
                    if let Ok(bars) = out {
                        draw_bars(context, &bars);
                    }
                }
                ProjectionMode::Line => {
                    if let Ok(segments) = chart.project_line_segments() {
                        draw_line_segments(context, &segments);
                    }
                }
                ProjectionMode::Area => {
                    let out = if overscan.get() {
                        chart.project_visible_area_geometry_with_overscan(0.06)
                    } else {
                        chart.project_visible_area_geometry()
                    };
                    if let Ok(area) = out {
                        draw_area(context, &area);
                    }
                }
                ProjectionMode::Baseline => {
                    let out = if overscan.get() {
                        chart.project_visible_baseline_geometry_with_overscan(baseline.get(), 0.06)
                    } else {
                        chart.project_visible_baseline_geometry(baseline.get())
                    };
                    if let Ok(geo) = out {
                        draw_baseline(context, &geo);
                    }
                }
                ProjectionMode::Histogram => {
                    let out = if overscan.get() {
                        chart.project_visible_histogram_bars_with_overscan(
                            8.0,
                            baseline.get(),
                            0.06,
                        )
                    } else {
                        chart.project_visible_histogram_bars(8.0, baseline.get())
                    };
                    if let Ok(bars) = out {
                        draw_histogram(context, &bars);
                    }
                }
                ProjectionMode::Markers => {
                    if let Ok(candles) = chart.project_visible_candles(6.0) {
                        draw_candles(context, &candles);
                    }
                    let out = if overscan.get() {
                        chart.project_visible_markers_on_candles_with_overscan(
                            markers.borrow().as_slice(),
                            0.06,
                            MarkerPlacementConfig::default(),
                        )
                    } else {
                        chart.project_visible_markers_on_candles(
                            markers.borrow().as_slice(),
                            MarkerPlacementConfig::default(),
                        )
                    };
                    if let Ok(placed) = out {
                        draw_markers(context, &placed);
                    }
                }
            }
        }
    });

    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let symbol_combo = gtk::ComboBoxText::new();
    for symbol in ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT"] {
        symbol_combo.append(Some(symbol), symbol);
    }
    symbol_combo.set_active_id(Some("BTCUSDT"));

    let interval_combo = gtk::ComboBoxText::new();
    for interval in ["1m", "5m", "15m", "1h", "4h", "1d"] {
        interval_combo.append(Some(interval), interval);
    }
    interval_combo.set_active_id(Some("1h"));

    let limit_spin = gtk::SpinButton::with_range(150.0, 1200.0, 50.0);
    limit_spin.set_value(420.0);

    let reload_button = gtk::Button::with_label("Reload Binance");

    let mode_combo = gtk::ComboBoxText::new();
    for value in ProjectionMode::ALL {
        mode_combo.append(Some(value.id()), value.label());
    }
    mode_combo.set_active_id(Some(ProjectionMode::Candles.id()));

    let overscan_toggle = gtk::CheckButton::with_label("Use overscan");
    overscan_toggle.set_active(true);

    let baseline_scale =
        gtk::Scale::with_range(gtk::Orientation::Horizontal, price_min, price_max, 0.1);
    baseline_scale.set_value((price_min + price_max) * 0.5);
    baseline_scale.set_hexpand(true);

    let fit_button = gtk::Button::with_label("Fit + Autoscale");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&gtk::Label::new(Some("Symbol")));
    controls.append(&symbol_combo);
    controls.append(&gtk::Label::new(Some("Interval")));
    controls.append(&interval_combo);
    controls.append(&gtk::Label::new(Some("Limit")));
    controls.append(&limit_spin);
    controls.append(&reload_button);
    controls.append(&mode_combo);
    controls.append(&overscan_toggle);
    controls.append(&gtk::Label::new(Some("Baseline")));
    controls.append(&baseline_scale);
    controls.append(&fit_button);

    {
        let mode = Rc::clone(&mode);
        let drawing_area = drawing_area.clone();
        mode_combo.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                mode.set(ProjectionMode::from_id(id.as_str()));
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
        fit_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.fit_time_to_data(TimeScaleTuning::default());
                let _ = chart.autoscale_price_from_candles();
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let markers = Rc::clone(&markers);
        let symbol_combo = symbol_combo.clone();
        let interval_combo = interval_combo.clone();
        let limit_spin = limit_spin.clone();
        reload_button.connect_clicked(move |_| {
            let symbol = symbol_combo
                .active_id()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "BTCUSDT".to_owned());
            let interval = interval_combo
                .active_id()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "1h".to_owned());
            let limit = limit_spin.value().round() as u16;

            let data = match binance::fetch_market_data(&symbol, &interval, limit) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("reload error: {err}");
                    binance::fallback_market_data(&symbol, &interval)
                }
            };

            if let Ok(mut chart) = engine.try_borrow_mut() {
                chart.set_data(data.close_points.clone());
                chart.set_candles(data.candles.clone());
                let _ = chart.fit_time_to_data(TimeScaleTuning::default());
                let _ = chart.autoscale_price_from_candles();
            }

            *markers.borrow_mut() = build_markers(&data.candles);
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
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(180), move || {
            if let Ok(chart) = engine.try_borrow() {
                let visible_points = chart.visible_points().len();
                let visible_candles = chart.visible_candles().len();
                let line = chart
                    .project_line_segments()
                    .map(|value| value.len())
                    .unwrap_or(0);
                let candles = chart
                    .project_visible_candles(8.0)
                    .map(|value| value.len())
                    .unwrap_or(0);
                let bars = chart
                    .project_visible_bars(8.0)
                    .map(|value| value.len())
                    .unwrap_or(0);

                status_label.set_text(&format!(
                    "projection={} overscan={} visible(points={},candles={}) counts(line={},candles={},bars={})",
                    mode.get().label(),
                    overscan.get(),
                    visible_points,
                    visible_candles,
                    line,
                    candles,
                    bars,
                ));
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let instructions = gtk::Label::new(Some(
        "Projection probe con Binance real: inspecciona geometría de project_visible_* y overlays sobre datos de mercado.",
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
        .title("chart-rs Binance Projection Probe")
        .default_width(1320)
        .default_height(880)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn build_markers(candles: &[chart_rs::core::OhlcBar]) -> Vec<SeriesMarker> {
    candles
        .iter()
        .enumerate()
        .step_by(22)
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
                .with_priority((index % 7) as i32)
        })
        .collect()
}

fn draw_line_segments(context: &gtk::cairo::Context, segments: &[LineSegment]) {
    context.set_source_rgba(0.15, 0.67, 0.98, 0.95);
    context.set_line_width(1.8);
    for segment in segments {
        context.move_to(segment.x1, segment.y1);
        context.line_to(segment.x2, segment.y2);
    }
    let _ = context.stroke();
}

fn draw_candles(context: &gtk::cairo::Context, candles: &[CandleGeometry]) {
    for candle in candles {
        let (r, g, b) = if candle.is_bullish {
            (0.00, 0.72, 0.42)
        } else {
            (0.90, 0.25, 0.24)
        };

        context.set_source_rgba(r, g, b, 0.95);
        context.move_to(candle.center_x, candle.wick_top);
        context.line_to(candle.center_x, candle.wick_bottom);
        let _ = context.stroke();

        context.set_source_rgba(r, g, b, 0.34);
        context.rectangle(
            candle.body_left,
            candle.body_top,
            (candle.body_right - candle.body_left).max(1.0),
            (candle.body_bottom - candle.body_top).max(1.0),
        );
        let _ = context.fill_preserve();

        context.set_source_rgba(r, g, b, 0.98);
        let _ = context.stroke();
    }
}

fn draw_bars(context: &gtk::cairo::Context, bars: &[BarGeometry]) {
    context.set_source_rgba(0.93, 0.95, 0.98, 0.9);
    context.set_line_width(1.2);
    for bar in bars {
        context.move_to(bar.center_x, bar.high_y);
        context.line_to(bar.center_x, bar.low_y);
        context.move_to(bar.open_x, bar.open_y);
        context.line_to(bar.center_x, bar.open_y);
        context.move_to(bar.center_x, bar.close_y);
        context.line_to(bar.close_x, bar.close_y);
    }
    let _ = context.stroke();
}

fn draw_area(context: &gtk::cairo::Context, geometry: &AreaGeometry) {
    if geometry.fill_polygon.is_empty() {
        return;
    }

    context.set_source_rgba(0.05, 0.64, 0.33, 0.22);
    if let Some(first) = geometry.fill_polygon.first() {
        context.move_to(first.x, first.y);
        for point in &geometry.fill_polygon[1..] {
            context.line_to(point.x, point.y);
        }
        context.close_path();
        let _ = context.fill();
    }

    context.set_source_rgba(0.04, 0.80, 0.46, 0.95);
    context.set_line_width(1.7);
    if let Some(first) = geometry.line_points.first() {
        context.move_to(first.x, first.y);
        for point in &geometry.line_points[1..] {
            context.line_to(point.x, point.y);
        }
        let _ = context.stroke();
    }
}

fn draw_baseline(context: &gtk::cairo::Context, geometry: &BaselineGeometry) {
    if geometry.line_points.is_empty() {
        return;
    }

    if let Some(first) = geometry.above_fill_polygon.first() {
        context.set_source_rgba(0.02, 0.71, 0.44, 0.20);
        context.move_to(first.x, first.y);
        for point in &geometry.above_fill_polygon[1..] {
            context.line_to(point.x, point.y);
        }
        context.close_path();
        let _ = context.fill();
    }

    if let Some(first) = geometry.below_fill_polygon.first() {
        context.set_source_rgba(0.92, 0.29, 0.24, 0.20);
        context.move_to(first.x, first.y);
        for point in &geometry.below_fill_polygon[1..] {
            context.line_to(point.x, point.y);
        }
        context.close_path();
        let _ = context.fill();
    }

    context.set_source_rgba(0.96, 0.80, 0.38, 0.95);
    context.move_to(0.0, geometry.baseline_y);
    context.line_to(20_000.0, geometry.baseline_y);
    let _ = context.stroke();
}

fn draw_histogram(context: &gtk::cairo::Context, bars: &[HistogramBar]) {
    context.set_source_rgba(0.22, 0.58, 0.97, 0.55);
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

fn draw_markers(context: &gtk::cairo::Context, markers: &[chart_rs::extensions::PlacedMarker]) {
    context.set_font_size(11.0);

    for marker in markers {
        let (r, g, b) = match marker.side {
            MarkerSide::Above => (0.20, 0.62, 0.97),
            MarkerSide::Below => (0.95, 0.43, 0.24),
            MarkerSide::Center => (0.05, 0.75, 0.40),
        };

        context.set_source_rgba(r, g, b, 0.95);
        context.arc(marker.x, marker.y, 4.0, 0.0, std::f64::consts::TAU);
        let _ = context.fill();

        if let Some(label) = &marker.label {
            context.set_source_rgba(0.08, 0.09, 0.12, 0.88);
            context.rectangle(label.left_px, label.top_px, label.width_px, label.height_px);
            let _ = context.fill();

            context.set_source_rgba(1.0, 1.0, 1.0, 0.95);
            context.move_to(label.left_px + 4.0, label.top_px + label.height_px - 3.0);
            let _ = context.show_text(&label.text);
        }
    }
}

#[allow(dead_code)]
fn _consume_frame(_frame: &RenderFrame) {
    let _ = Color::rgb(0.0, 0.0, 0.0);
}
