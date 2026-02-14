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
        .application_id("rs.chart.examples.binance.logical-inspector")
        .build();

    app.connect_activate(|app| {
        let engine = match build_engine_with_binance_candles("ADAUSDT", "15m", 600, 1280, 760) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("failed to initialize engine from Binance: {err}");
                return;
            }
        };

        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let info = gtk::Label::new(Some("move pointer to inspect nearest filled logical slot"));
        info.set_xalign(0.0);

        let inspector_motion = gtk::EventControllerMotion::new();
        {
            let adapter = Rc::clone(&adapter);
            let info = info.clone();
            inspector_motion.connect_motion(move |_, x, _| {
                match adapter.nearest_filled_logical_slot_at_pixel(x) {
                    Ok(Some(slot)) => info.set_text(&format!(
                        "source={:?} slot={} logical_index={:.4} time={}",
                        slot.source, slot.slot, slot.logical_index, slot.time
                    )),
                    Ok(None) => info.set_text("no filled slot at this pixel"),
                    Err(err) => info.set_text(&format!("slot inspect error: {err}")),
                }
            });
        }
        adapter.drawing_area().add_controller(inspector_motion);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 6);
        layout.append(adapter.drawing_area());
        layout.append(&info);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance ADAUSDT logical-slot inspector")
            .default_width(1280)
            .default_height(800)
            .build();
        window.set_child(Some(&layout));
        window.present();
    });

    let _ = app.run();
}

#[cfg(not(feature = "gtk4-adapter"))]
fn main() {
    println!("run with: cargo run --features desktop --example gtk_binance_logical_inspector");
}
