#[cfg(feature = "gtk4-adapter")]
#[path = "shared/binance.rs"]
mod gtk_binance_support;

#[cfg(feature = "gtk4-adapter")]
fn main() {
    use std::rc::Rc;

    use gtk4 as gtk;
    use gtk4::glib;
    use gtk4::prelude::*;

    use gtk_binance_support::{
        build_engine_with_binance_candles, fetch_binance_klines, install_default_interaction,
        klines_to_ohlc,
    };

    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.binance.live")
        .build();

    app.connect_activate(|app| {
        let engine = match build_engine_with_binance_candles("BTCUSDT", "1m", 600, 1280, 760) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("failed to initialize engine from Binance: {err}");
                return;
            }
        };

        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let status = gtk::Label::new(Some("Polling Binance every 3s..."));
        status.set_xalign(0.0);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 6);
        layout.append(adapter.drawing_area());
        layout.append(&status);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance BTCUSDT 1m (live poll)")
            .default_width(1280)
            .default_height(800)
            .build();
        window.set_child(Some(&layout));
        window.present();

        let adapter_timer = Rc::clone(&adapter);
        let status_timer = status.clone();
        glib::timeout_add_seconds_local(3, move || {
            match fetch_binance_klines("BTCUSDT", "1m", 2).and_then(|k| klines_to_ohlc(&k)) {
                Ok(ohlc) => {
                    let update_result = adapter_timer.update_engine(|engine| {
                        for candle in ohlc {
                            engine.update_candle(candle)?;
                        }
                        engine.autoscale_price_from_visible_candles()?;
                        Ok(())
                    });
                    if let Err(err) = update_result {
                        status_timer.set_text(&format!("engine update error: {err}"));
                    } else {
                        status_timer.set_text("last update OK");
                    }
                }
                Err(err) => status_timer.set_text(&format!("binance poll error: {err}")),
            }
            glib::ControlFlow::Continue
        });
    });

    let _ = app.run();
}

#[cfg(not(feature = "gtk4-adapter"))]
fn main() {
    println!("run with: cargo run --features desktop --example gtk_binance_live_poll");
}
