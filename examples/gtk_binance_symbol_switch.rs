#[cfg(feature = "gtk4-adapter")]
#[path = "shared/binance.rs"]
mod gtk_binance_support;

#[cfg(feature = "gtk4-adapter")]
fn main() {
    use std::rc::Rc;

    use gtk4 as gtk;
    use gtk4::prelude::*;

    use gtk_binance_support::{
        build_engine_with_binance_candles, fetch_binance_klines, install_default_interaction,
        klines_to_ohlc,
    };

    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.binance.symbol-switch")
        .build();

    app.connect_activate(|app| {
        let engine = match build_engine_with_binance_candles("BTCUSDT", "15m", 600, 1280, 760) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("failed to initialize engine from Binance: {err}");
                return;
            }
        };
        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let combo = gtk::DropDown::from_strings(&["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"]);
        combo.set_selected(0);
        let status = gtk::Label::new(Some("symbol: BTCUSDT"));
        status.set_xalign(0.0);

        {
            let adapter = Rc::clone(&adapter);
            let status = status.clone();
            combo.connect_selected_notify(move |c| {
                let Some(model) = c.model() else {
                    return;
                };
                let idx = c.selected();
                let Some(item) = model.item(idx) else {
                    return;
                };
                let Ok(string_obj) = item.downcast::<gtk::StringObject>() else {
                    return;
                };
                let symbol = string_obj.string().to_string();
                status.set_text(&format!("loading {symbol} ..."));

                match fetch_binance_klines(&symbol, "15m", 600).and_then(|k| klines_to_ohlc(&k)) {
                    Ok(candles) => {
                        let update = adapter.update_engine(|engine| {
                            engine.set_candles(candles);
                            engine.autoscale_price_from_candles()?;
                            engine.fit_time_to_data(chart_rs::core::TimeScaleTuning::default())?;
                            Ok(())
                        });
                        if let Err(err) = update {
                            status.set_text(&format!("engine update error: {err}"));
                        } else {
                            status.set_text(&format!("symbol: {symbol}"));
                        }
                    }
                    Err(err) => status.set_text(&format!("binance error: {err}")),
                }
            });
        }

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.append(&gtk::Label::new(Some("Symbol:")));
        header.append(&combo);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 6);
        layout.append(&header);
        layout.append(adapter.drawing_area());
        layout.append(&status);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance symbol switch")
            .default_width(1280)
            .default_height(820)
            .build();
        window.set_child(Some(&layout));
        window.present();
    });

    let _ = app.run();
}

#[cfg(not(feature = "gtk4-adapter"))]
fn main() {
    println!("run with: cargo run --features desktop --example gtk_binance_symbol_switch");
}
