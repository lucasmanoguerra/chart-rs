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
        .application_id("rs.chart.examples.binance.crosshair")
        .build();

    app.connect_activate(|app| {
        let engine = match build_engine_with_binance_candles("BNBUSDT", "1m", 500, 1280, 760) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("failed to initialize engine from Binance: {err}");
                return;
            }
        };

        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let info = gtk::Label::new(Some(
            "Crosshair mode: Magnet (keys: M=Magnet, N=Normal, H=Hidden)",
        ));
        info.set_xalign(0.0);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 6);
        layout.append(adapter.drawing_area());
        layout.append(&info);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance BNBUSDT crosshair modes")
            .default_width(1280)
            .default_height(800)
            .build();
        window.set_child(Some(&layout));

        let key = gtk::EventControllerKey::new();
        {
            let adapter = Rc::clone(&adapter);
            let info = info.clone();
            key.connect_key_pressed(move |_, keyval, _, _| {
                let mut set_label = None::<&str>;
                let _ = adapter.update_engine(|engine| {
                    match keyval {
                        gdk::Key::m | gdk::Key::M => {
                            engine.set_crosshair_mode(chart_rs::api::CrosshairMode::Magnet);
                            set_label = Some("Crosshair mode: Magnet");
                        }
                        gdk::Key::n | gdk::Key::N => {
                            engine.set_crosshair_mode(chart_rs::api::CrosshairMode::Normal);
                            set_label = Some("Crosshair mode: Normal");
                        }
                        gdk::Key::h | gdk::Key::H => {
                            engine.set_crosshair_mode(chart_rs::api::CrosshairMode::Hidden);
                            set_label = Some("Crosshair mode: Hidden");
                        }
                        _ => {}
                    }
                    Ok(())
                });
                if let Some(text) = set_label {
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
    println!("run with: cargo run --features desktop --example gtk_binance_crosshair_modes");
}
