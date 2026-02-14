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
        .application_id("rs.chart.examples.binance.axis-interactions")
        .build();

    app.connect_activate(|app| {
        let engine = match build_engine_with_binance_candles("XRPUSDT", "5m", 700, 1280, 760) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("failed to initialize engine from Binance: {err}");
                return;
            }
        };
        let adapter = Rc::new(chart_rs::platform_gtk::GtkChartAdapter::new(engine));
        install_default_interaction(Rc::clone(&adapter));

        let info = gtk::Label::new(Some(
            "Drag right axis: scale price | Drag bottom axis: scale time | Double click axis: reset",
        ));
        info.set_xalign(0.0);

        let click = gtk::GestureClick::new();
        {
            let adapter = Rc::clone(&adapter);
            click.connect_pressed(move |_, n_press, x, y| {
                if n_press != 2 {
                    return;
                }
                let _ = adapter.update_engine(|engine| {
                    let style = engine.render_style();
                    let viewport = engine.viewport();
                    let plot_right = viewport.width as f64 - style.price_axis_width_px;
                    let plot_bottom = viewport.height as f64 - style.time_axis_height_px;

                    if x >= plot_right {
                        let _ = engine.axis_double_click_reset_price_scale()?;
                    } else if y >= plot_bottom {
                        let _ = engine.axis_double_click_reset_time_scale()?;
                    }
                    Ok(())
                });
            });
        }
        adapter.drawing_area().add_controller(click);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 6);
        layout.append(adapter.drawing_area());
        layout.append(&info);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("chart-rs | Binance XRPUSDT axis interactions")
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
    println!("run with: cargo run --features desktop --example gtk_binance_axis_interactions");
}
