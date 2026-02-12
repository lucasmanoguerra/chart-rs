#[path = "shared/mod.rs"]
mod shared;

use std::cell::RefCell;
use std::rc::Rc;

use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, LastPriceLabelBoxWidthMode,
    PriceAxisLabelConfig, PriceAxisLabelPolicy, PriceScaleMarginBehavior, RenderStyle,
    TimeAxisLabelConfig, TimeAxisLabelPolicy,
};
use chart_rs::core::{CandleGeometry, TimeScaleTuning, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{CairoRenderer, Color, LineStrokeStyle};
use gtk4 as gtk;
use gtk4::prelude::*;

use shared::binance;

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_binance_axis_scale_interactions")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let data = match binance::fetch_market_data("BTCUSDT", "5m", 420) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("binance fetch failed, using fallback dataset: {err}");
            binance::fallback_market_data("BTCUSDT", "5m")
        }
    };

    let engine = match build_engine(&data) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("failed to initialize axis interaction example: {err}");
            return;
        }
    };
    let engine = Rc::new(RefCell::new(engine));

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_draw_func({
        let engine = Rc::clone(&engine);
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

            if let Ok(candles) = chart.project_visible_candles_with_overscan(8.0, 0.03) {
                draw_mono_candles(context, &candles);
            }
        }
    });

    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("chart-rs Axis Scale Interaction Parity")
        .default_width(1365)
        .default_height(768)
        .build();
    window.set_child(Some(&drawing_area));
    window.present();
}

fn build_engine(data: &binance::MarketData) -> chart_rs::ChartResult<ChartEngine<CairoRenderer>> {
    let (time_start, time_end) = binance::data_time_range(data);
    let (price_min, price_max) = binance::data_price_range(data);

    let mut renderer = CairoRenderer::new(1365, 768)?;
    renderer.set_clear_color(Color::rgb(0.70, 0.70, 0.70))?;

    let config = ChartEngineConfig::new(Viewport::new(1365, 768), time_start, time_end)
        .with_price_domain(price_min, price_max)
        .with_crosshair_mode(CrosshairMode::Normal);
    let mut engine = ChartEngine::new(renderer, config)?;

    engine.set_candles(data.candles.clone());
    engine.set_time_axis_label_config(TimeAxisLabelConfig {
        locale: AxisLabelLocale::EsEs,
        policy: TimeAxisLabelPolicy::UtcAdaptive,
        ..TimeAxisLabelConfig::default()
    })?;
    engine.set_price_axis_label_config(PriceAxisLabelConfig {
        locale: AxisLabelLocale::EsEs,
        policy: PriceAxisLabelPolicy::FixedDecimals { precision: 2 },
        ..PriceAxisLabelConfig::default()
    })?;
    engine.set_price_scale_margin_behavior(PriceScaleMarginBehavior {
        top_margin_ratio: 0.02,
        bottom_margin_ratio: 0.03,
    })?;
    engine.fit_time_to_data(TimeScaleTuning::default())?;
    let (_, full_end) = engine.time_full_range();
    let tail_start = full_end - 18.0 * 3_600.0;
    let _ = engine.set_time_visible_range(tail_start, full_end);
    engine.autoscale_price_from_candles()?;

    let style = RenderStyle {
        series_line_color: Color::rgba(0.25, 0.25, 0.25, 0.0),
        grid_line_color: Color::rgba(0.22, 0.22, 0.22, 0.18),
        price_axis_grid_line_color: Color::rgba(0.22, 0.22, 0.22, 0.18),
        major_grid_line_color: Color::rgba(0.20, 0.20, 0.20, 0.30),
        axis_border_color: Color::rgba(0.16, 0.16, 0.16, 0.55),
        axis_label_color: Color::rgba(0.14, 0.14, 0.14, 0.98),
        time_axis_label_color: Color::rgba(0.14, 0.14, 0.14, 0.98),
        major_time_label_color: Color::rgba(0.14, 0.14, 0.14, 0.98),
        crosshair_line_color: Color::rgba(0.22, 0.22, 0.22, 0.55),
        crosshair_horizontal_line_color: Some(Color::rgba(0.22, 0.22, 0.22, 0.55)),
        crosshair_vertical_line_color: Some(Color::rgba(0.22, 0.22, 0.22, 0.55)),
        crosshair_line_style: LineStrokeStyle::Dashed,
        crosshair_time_label_color: Color::rgb(1.0, 1.0, 1.0),
        crosshair_price_label_color: Color::rgb(1.0, 1.0, 1.0),
        show_crosshair_time_label_box: true,
        show_crosshair_price_label_box: true,
        crosshair_time_label_box_color: Some(Color::rgba(0.10, 0.10, 0.10, 0.97)),
        crosshair_price_label_box_color: Some(Color::rgba(0.10, 0.10, 0.10, 0.97)),
        crosshair_time_label_box_text_color: Some(Color::rgb(1.0, 1.0, 1.0)),
        crosshair_price_label_box_text_color: Some(Color::rgb(1.0, 1.0, 1.0)),
        show_crosshair_time_label_box_border: false,
        show_crosshair_price_label_box_border: false,
        last_price_line_color: Color::rgba(0.18, 0.18, 0.18, 0.55),
        last_price_label_color: Color::rgb(1.0, 1.0, 1.0),
        show_last_price_label_box: true,
        last_price_label_box_use_marker_color: false,
        last_price_label_box_color: Color::rgba(0.10, 0.10, 0.10, 0.97),
        last_price_label_box_text_color: Color::rgb(1.0, 1.0, 1.0),
        last_price_label_box_auto_text_contrast: false,
        last_price_label_box_width_mode: LastPriceLabelBoxWidthMode::FitText,
        last_price_label_box_border_width_px: 1.0,
        last_price_label_box_border_color: Color::rgba(0.35, 0.35, 0.35, 0.95),
        price_axis_width_px: 88.0,
        time_axis_height_px: 26.0,
        show_major_time_labels: false,
        ..engine.render_style()
    };
    engine.set_render_style(style)?;

    Ok(engine)
}

fn draw_mono_candles(context: &gtk::cairo::Context, candles: &[CandleGeometry]) {
    for candle in candles {
        let (fill_r, fill_g, fill_b, fill_a, stroke_r, stroke_g, stroke_b) = if candle.is_bullish {
            (0.86, 0.86, 0.86, 0.96, 0.20, 0.20, 0.20)
        } else {
            (0.08, 0.08, 0.08, 0.96, 0.08, 0.08, 0.08)
        };

        context.set_source_rgba(stroke_r, stroke_g, stroke_b, 0.95);
        context.set_line_width(1.0);
        context.move_to(candle.center_x, candle.wick_top);
        context.line_to(candle.center_x, candle.wick_bottom);
        let _ = context.stroke();

        context.set_source_rgba(fill_r, fill_g, fill_b, fill_a);
        context.rectangle(
            candle.body_left,
            candle.body_top,
            (candle.body_right - candle.body_left).max(1.0),
            (candle.body_bottom - candle.body_top).max(1.0),
        );
        let _ = context.fill_preserve();

        context.set_source_rgba(stroke_r, stroke_g, stroke_b, 0.95);
        context.set_line_width(1.0);
        let _ = context.stroke();
    }
}
