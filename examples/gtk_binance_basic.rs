#[cfg(feature = "gtk4-adapter")]
#[path = "shared/binance.rs"]
mod gtk_binance_support;

#[cfg(feature = "gtk4-adapter")]
fn main() {
    use std::rc::Rc;

    use gtk4 as gtk;
    use gtk4::prelude::*;

    use gtk_binance_support::{build_engine_with_binance_candles, install_default_interaction};

    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.binance.basic")
        .build();

    app.connect_activate(|app| {
        let engine = match build_engine_with_binance_candles("BTCUSDT", "1m", 500, 1280, 760) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("failed to initialize engine from Binance: {err}");
                return;
            }
        };

        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance BTCUSDT 1m")
            .default_width(1280)
            .default_height(760)
            .build();
        window.set_child(Some(adapter.drawing_area()));
        window.present();
    });

    let _ = app.run();
}

#[cfg(not(feature = "gtk4-adapter"))]
fn main() {
    println!("run with: cargo run --features desktop --example gtk_binance_basic");
}
