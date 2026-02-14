#[cfg(feature = "gtk4-adapter")]
#[path = "shared/binance.rs"]
mod gtk_binance_support;

#[cfg(feature = "gtk4-adapter")]
fn main() {
    use std::rc::Rc;

    use gtk4 as gtk;
    use gtk4::gdk;
    use gtk4::prelude::*;

    use gtk_binance_support::{build_engine_with_binance_candles, install_default_interaction};

    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.binance.price-scale-modes")
        .build();

    app.connect_activate(|app| {
        let engine = match build_engine_with_binance_candles("SOLUSDT", "15m", 700, 1280, 760) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("failed to initialize engine from Binance: {err}");
                return;
            }
        };
        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let info = gtk::Label::new(Some(
            "Price scale: Linear (keys: 1=Linear, 2=Percentage, 3=IndexedTo100)",
        ));
        info.set_xalign(0.0);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 6);
        layout.append(adapter.drawing_area());
        layout.append(&info);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance SOLUSDT price scale modes")
            .default_width(1280)
            .default_height(800)
            .build();
        window.set_child(Some(&layout));

        let key = gtk::EventControllerKey::new();
        {
            let adapter = Rc::clone(&adapter);
            let info = info.clone();
            key.connect_key_pressed(move |_, keyval, _, _| {
                let mut label = None::<&str>;
                let _ = adapter.update_engine(|engine| {
                    use chart_rs::core::PriceScaleMode;
                    let mode = match keyval {
                        gdk::Key::_1 => Some(PriceScaleMode::Linear),
                        gdk::Key::_2 => Some(PriceScaleMode::Percentage),
                        gdk::Key::_3 => Some(PriceScaleMode::IndexedTo100),
                        _ => None,
                    };
                    if let Some(mode) = mode {
                        engine.set_price_scale_mode(mode)?;
                        label = Some(match mode {
                            PriceScaleMode::Linear => "Price scale: Linear",
                            PriceScaleMode::Percentage => "Price scale: Percentage",
                            PriceScaleMode::IndexedTo100 => "Price scale: IndexedTo100",
                            PriceScaleMode::Log => "Price scale: Log",
                        });
                        engine.autoscale_price_from_visible_candles()?;
                    }
                    Ok(())
                });
                if let Some(text) = label {
                    info.set_text(text);
                }
                gtk::glib::Propagation::Proceed
            });
        }
        window.add_controller(key);
        window.present();
    });

    let _ = app.run();
}

#[cfg(not(feature = "gtk4-adapter"))]
fn main() {
    println!("run with: cargo run --features desktop --example gtk_binance_price_scale_modes");
}
