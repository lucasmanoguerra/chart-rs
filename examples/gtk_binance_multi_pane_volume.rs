#[cfg(feature = "gtk4-adapter")]
#[path = "shared/binance.rs"]
mod gtk_binance_support;

#[cfg(feature = "gtk4-adapter")]
fn main() {
    use std::rc::Rc;

    use gtk4 as gtk;
    use gtk4::prelude::*;

    use gtk_binance_support::{
        fetch_binance_klines, install_default_interaction, klines_to_ohlc, klines_to_volume_points,
    };

    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.binance.multi-pane")
        .build();

    app.connect_activate(|app| {
        let klines = match fetch_binance_klines("ETHUSDT", "5m", 700) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("binance fetch error: {err}");
                return;
            }
        };
        let candles = match klines_to_ohlc(&klines) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("ohlc conversion error: {err}");
                return;
            }
        };
        let volumes = klines_to_volume_points(&klines);
        let (min_t, max_t) = match (klines.first(), klines.last()) {
            (Some(a), Some(b)) => (a.open_time_sec, b.open_time_sec),
            _ => {
                eprintln!("no kline data");
                return;
            }
        };

        let renderer = match chart_rs::render::CairoRenderer::new(1360, 820) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("renderer init error: {err}");
                return;
            }
        };
        let config = chart_rs::api::ChartEngineConfig::new(
            chart_rs::core::Viewport::new(1360, 820),
            min_t,
            max_t,
        )
        .with_price_domain(0.0, 1.0);
        let mut engine = match chart_rs::api::ChartEngine::new(renderer, config) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("engine init error: {err}");
                return;
            }
        };

        let volume_pane = match engine.create_pane(1.0) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("pane create error: {err}");
                return;
            }
        };
        let _ = engine.set_points_pane(volume_pane);
        engine.set_candles(candles);
        engine.set_data(volumes);
        let _ = engine.autoscale_price_from_candles();
        let _ = engine.fit_time_to_data(chart_rs::core::TimeScaleTuning::default());

        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance ETHUSDT 5m (candles + volume pane)")
            .default_width(1360)
            .default_height(820)
            .build();
        window.set_child(Some(adapter.drawing_area()));
        window.present();
    });

    let _ = app.run();
}

#[cfg(not(feature = "gtk4-adapter"))]
fn main() {
    println!("run with: cargo run --features desktop --example gtk_binance_multi_pane_volume");
}
