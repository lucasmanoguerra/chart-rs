use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceAxisLabelPolicy, RenderStyle, TimeAxisLabelConfig, TimeAxisLabelPolicy,
    TimeAxisSessionConfig, TimeAxisTimeZone,
};
use chart_rs::core::{
    DataPoint, LinearScale, OhlcBar, PriceScale, PriceScaleMode, TimeScale, Viewport,
    points_in_time_window, project_area_geometry, project_bars, project_baseline_geometry,
    project_candles, project_histogram_bars, project_line_segments,
};
use chart_rs::extensions::{
    ChartPlugin, MarkerPlacementConfig, MarkerPosition, PluginContext, PluginEvent, SeriesMarker,
    place_markers_on_candles,
};
use chart_rs::interaction::{CrosshairMode, KineticPanConfig};
use chart_rs::render::NullRenderer;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_linear_scale_round_trip(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let scale = LinearScale::new(0.0, 10_000.0).expect("valid scale");

    c.bench_function("linear_scale_round_trip", |b| {
        b.iter(|| {
            let px = scale
                .domain_to_pixel(4_321.123, viewport)
                .expect("to pixel");
            let _ = scale.pixel_to_domain(px, viewport).expect("from pixel");
        })
    });
}

fn bench_candle_projection_10k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 10_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 2_500.0).expect("valid price scale");

    let bars: Vec<OhlcBar> = (0..10_000)
        .map(|i| {
            let t = i as f64;
            let base = 100.0 + t * 0.05;
            let open = base;
            let close = if i % 2 == 0 { base + 1.0 } else { base - 1.0 };
            let low = open.min(close) - 0.75;
            let high = open.max(close) + 0.75;
            OhlcBar::new(t, open, high, low, close).expect("valid generated bar")
        })
        .collect();

    c.bench_function("candle_projection_10k", |b| {
        b.iter(|| {
            let _ = project_candles(
                black_box(&bars),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
                black_box(7.0),
            )
            .expect("projection should succeed");
        })
    });
}

fn bench_bar_projection_10k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 10_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 2_500.0).expect("valid price scale");

    let bars: Vec<OhlcBar> = (0..10_000)
        .map(|i| {
            let t = i as f64;
            let base = 100.0 + t * 0.05;
            let open = base;
            let close = if i % 2 == 0 { base + 1.0 } else { base - 1.0 };
            let low = open.min(close) - 0.75;
            let high = open.max(close) + 0.75;
            OhlcBar::new(t, open, high, low, close).expect("valid generated bar")
        })
        .collect();

    c.bench_function("bar_projection_10k", |b| {
        b.iter(|| {
            let _ = project_bars(
                black_box(&bars),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
                black_box(7.0),
            )
            .expect("bar projection should succeed");
        })
    });
}

fn bench_line_projection_20k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 20_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 5_000.0).expect("valid price scale");

    let points: Vec<DataPoint> = (0..20_000)
        .map(|i| {
            let t = i as f64;
            let y = 1_000.0 + (t * 0.07).sin() * 250.0 + t * 0.02;
            DataPoint::new(t, y)
        })
        .collect();

    c.bench_function("line_projection_20k", |b| {
        b.iter(|| {
            let _ = project_line_segments(
                black_box(&points),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
            )
            .expect("line projection should succeed");
        })
    });
}

fn bench_area_projection_20k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 20_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 5_000.0).expect("valid price scale");

    let points: Vec<DataPoint> = (0..20_000)
        .map(|i| {
            let t = i as f64;
            let y = 1_000.0 + (t * 0.07).sin() * 250.0 + t * 0.02;
            DataPoint::new(t, y)
        })
        .collect();

    c.bench_function("area_projection_20k", |b| {
        b.iter(|| {
            let _ = project_area_geometry(
                black_box(&points),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
            )
            .expect("area projection should succeed");
        })
    });
}

fn bench_baseline_projection_20k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 20_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 5_000.0).expect("valid price scale");

    let points: Vec<DataPoint> = (0..20_000)
        .map(|i| {
            let t = i as f64;
            let y = 1_000.0 + (t * 0.07).sin() * 250.0 + t * 0.02;
            DataPoint::new(t, y)
        })
        .collect();

    c.bench_function("baseline_projection_20k", |b| {
        b.iter(|| {
            let _ = project_baseline_geometry(
                black_box(&points),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
                black_box(1_000.0),
            )
            .expect("baseline projection should succeed");
        })
    });
}

fn bench_histogram_projection_20k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 20_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 5_000.0).expect("valid price scale");

    let points: Vec<DataPoint> = (0..20_000)
        .map(|i| {
            let t = i as f64;
            let y = 1_000.0 + (t * 0.07).sin() * 250.0 + t * 0.02;
            DataPoint::new(t, y)
        })
        .collect();

    c.bench_function("histogram_projection_20k", |b| {
        b.iter(|| {
            let _ = project_histogram_bars(
                black_box(&points),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
                black_box(5.0),
                black_box(1_000.0),
            )
            .expect("histogram projection should succeed");
        })
    });
}

fn bench_visible_window_points_100k(c: &mut Criterion) {
    let points: Vec<DataPoint> = (0..100_000)
        .map(|i| {
            let x = i as f64;
            let y = (x * 0.02).sin() * 100.0 + x * 0.001;
            DataPoint::new(x, y)
        })
        .collect();

    c.bench_function("visible_window_points_100k", |b| {
        b.iter(|| {
            let _ =
                points_in_time_window(black_box(&points), black_box(45_000.0), black_box(55_000.0));
        })
    });
}

fn bench_marker_placement_5k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 5_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 10_000.0).expect("valid price scale");

    let candles: Vec<OhlcBar> = (0..5_000)
        .map(|i| {
            let t = i as f64;
            let open = 1_000.0 + t * 0.2;
            let close = if i % 2 == 0 { open + 2.0 } else { open - 2.0 };
            let low = open.min(close) - 1.0;
            let high = open.max(close) + 1.0;
            OhlcBar::new(t, open, high, low, close).expect("valid generated candle")
        })
        .collect();
    let markers: Vec<SeriesMarker> = (0..5_000)
        .map(|i| {
            let position = if i % 2 == 0 {
                MarkerPosition::AboveBar
            } else {
                MarkerPosition::BelowBar
            };
            SeriesMarker::new(format!("m-{i}"), i as f64, position)
                .with_text("marker")
                .with_priority(i % 10)
        })
        .collect();

    c.bench_function("marker_placement_5k", |b| {
        b.iter(|| {
            let _ = place_markers_on_candles(
                black_box(&markers),
                black_box(&candles),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
                black_box(MarkerPlacementConfig::default()),
            )
            .expect("marker placement should succeed");
        })
    });
}

fn bench_engine_snapshot_json_2k(c: &mut Criterion) {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 2_001.0)
        .with_price_domain(0.0, 2_500.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let bars: Vec<OhlcBar> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            let base = 400.0 + t * 0.03;
            let open = base;
            let close = if i % 2 == 0 { base + 2.0 } else { base - 2.0 };
            let low = open.min(close) - 1.0;
            let high = open.max(close) + 1.0;
            OhlcBar::new(t, open, high, low, close).expect("valid generated bar")
        })
        .collect();

    engine.set_series_metadata("series-id", "candles-main");
    engine.set_series_metadata("series-type", "candlestick");
    engine.set_candles(bars);

    c.bench_function("engine_snapshot_json_2k", |b| {
        b.iter(|| {
            let _ = engine
                .snapshot_json_pretty(black_box(7.0))
                .expect("snapshot json should succeed");
        })
    });
}

fn bench_plugin_dispatch_pointer_move(c: &mut Criterion) {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    for i in 0..20 {
        struct BenchPlugin {
            id: String,
        }
        impl ChartPlugin for BenchPlugin {
            fn id(&self) -> &str {
                &self.id
            }

            fn on_event(&mut self, _event: PluginEvent, _context: PluginContext) {}
        }
        engine
            .register_plugin(Box::new(BenchPlugin {
                id: format!("noop-{i}"),
            }))
            .expect("register plugin");
    }

    c.bench_function("plugin_dispatch_pointer_move_20_plugins", |b| {
        b.iter(|| {
            engine.pointer_move(black_box(400.0), black_box(300.0));
        })
    });
}

fn bench_crosshair_modes_pointer_move(c: &mut Criterion) {
    let points: Vec<DataPoint> = (0..5_000)
        .map(|i| {
            let t = i as f64;
            DataPoint::new(t, 1_000.0 + (t * 0.01).sin() * 100.0)
        })
        .collect();

    let mut engine_magnet = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 5_000.0)
            .with_price_domain(0.0, 2_000.0),
    )
    .expect("engine init");
    engine_magnet.set_data(points.clone());
    engine_magnet.set_crosshair_mode(CrosshairMode::Magnet);

    let mut engine_normal = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 5_000.0)
            .with_price_domain(0.0, 2_000.0),
    )
    .expect("engine init");
    engine_normal.set_data(points);
    engine_normal.set_crosshair_mode(CrosshairMode::Normal);

    c.bench_function("crosshair_pointer_move_magnet", |b| {
        b.iter(|| {
            engine_magnet.pointer_move(black_box(750.0), black_box(300.0));
        })
    });

    c.bench_function("crosshair_pointer_move_normal", |b| {
        b.iter(|| {
            engine_normal.pointer_move(black_box(750.0), black_box(300.0));
        })
    });
}

fn bench_wheel_zoom_step(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 5_000.0).with_price_domain(0.0, 1.0),
    )
    .expect("engine init");

    c.bench_function("wheel_zoom_step_pair", |b| {
        b.iter(|| {
            let _ = engine
                .wheel_zoom_time_visible(
                    black_box(-120.0),
                    black_box(800.0),
                    black_box(0.2),
                    black_box(1e-6),
                )
                .expect("wheel zoom in");
            let _ = engine
                .wheel_zoom_time_visible(
                    black_box(120.0),
                    black_box(800.0),
                    black_box(0.2),
                    black_box(1e-6),
                )
                .expect("wheel zoom out");
        })
    });
}

fn bench_wheel_pan_step(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 5_000.0).with_price_domain(0.0, 1.0),
    )
    .expect("engine init");

    c.bench_function("wheel_pan_step_pair", |b| {
        b.iter(|| {
            let _ = engine
                .wheel_pan_time_visible(black_box(120.0), black_box(0.1))
                .expect("wheel pan forward");
            let _ = engine
                .wheel_pan_time_visible(black_box(-120.0), black_box(0.1))
                .expect("wheel pan back");
        })
    });
}

fn bench_kinetic_pan_step(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 5_000.0).with_price_domain(0.0, 1.0),
    )
    .expect("engine init");
    engine
        .set_kinetic_pan_config(KineticPanConfig {
            decay_per_second: 0.85,
            stop_velocity_abs: 0.01,
        })
        .expect("set config");

    c.bench_function("kinetic_pan_step_active", |b| {
        b.iter(|| {
            engine
                .start_kinetic_pan(black_box(300.0))
                .expect("start kinetic");
            let _ = engine
                .step_kinetic_pan(black_box(0.016))
                .expect("step kinetic");
        })
    });
}

fn bench_render_frame_build_20k(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1920, 1080), 0.0, 20_000.0)
            .with_price_domain(-2_000.0, 2_000.0),
    )
    .expect("engine init");

    let points: Vec<DataPoint> = (0..20_000)
        .map(|i| {
            let t = i as f64;
            let y = (t * 0.01).sin() * 500.0 + t * 0.02 - 1_000.0;
            DataPoint::new(t, y)
        })
        .collect();
    engine.set_data(points);

    c.bench_function("render_frame_build_20k", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_render_axis_layout_narrow(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(220, 140), 0.0, 5_000.0).with_price_domain(0.0, 1.0),
    )
    .expect("engine init");

    let points: Vec<DataPoint> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            let y = 0.5 + (t * 0.02).sin() * 0.3;
            DataPoint::new(t, y)
        })
        .collect();
    engine.set_data(points);

    c.bench_function("render_axis_layout_narrow", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_time_axis_datetime_formatter(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(900, 420), 1_700_000_000.0, 1_700_010_000.0)
            .with_price_domain(0.0, 1.0),
    )
    .expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcDateTime {
                show_seconds: false,
            },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set formatter policy");

    c.bench_function("time_axis_datetime_formatter", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_time_axis_label_cache_hot(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(
            Viewport::new(900, 420),
            1_700_000_000.0,
            1_700_000_000.0 + 86_400.0,
        )
        .with_price_domain(0.0, 1.0),
    )
    .expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcAdaptive,
            ..TimeAxisLabelConfig::default()
        })
        .expect("set adaptive policy");
    engine.clear_time_label_cache();
    let _ = engine.build_render_frame().expect("warm cache");

    c.bench_function("time_axis_label_cache_hot", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_time_axis_session_timezone_formatter(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 1_704_205_800.0, 1_704_206_100.0)
            .with_price_domain(0.0, 1.0),
    )
    .expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcDateTime {
                show_seconds: false,
            },
            timezone: TimeAxisTimeZone::FixedOffsetMinutes { minutes: -300 },
            session: Some(TimeAxisSessionConfig {
                start_hour: 9,
                start_minute: 30,
                end_hour: 16,
                end_minute: 0,
            }),
        })
        .expect("set session+timezone policy");

    c.bench_function("time_axis_session_timezone_formatter", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_render_major_time_tick_styling(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 1_704_205_800.0, 1_704_206_100.0)
            .with_price_domain(0.0, 1.0),
    )
    .expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcDateTime {
                show_seconds: false,
            },
            timezone: TimeAxisTimeZone::FixedOffsetMinutes { minutes: -300 },
            session: Some(TimeAxisSessionConfig {
                start_hour: 9,
                start_minute: 30,
                end_hour: 16,
                end_minute: 0,
            }),
        })
        .expect("set session+timezone policy");
    engine
        .set_render_style(RenderStyle {
            major_grid_line_width: 2.0,
            major_time_label_font_size_px: 14.0,
            ..engine.render_style()
        })
        .expect("set major style");

    c.bench_function("render_major_time_tick_styling", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_price_axis_min_move_formatter(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 0.0, 1_000.0)
            .with_price_domain(99.0, 101.0),
    )
    .expect("engine init");
    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::MinMove {
                min_move: 0.01,
                trim_trailing_zeros: false,
            },
            ..PriceAxisLabelConfig::default()
        })
        .expect("set price-axis policy");

    c.bench_function("price_axis_min_move_formatter", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_price_axis_percentage_display(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 0.0, 1_000.0)
            .with_price_domain(95.0, 105.0),
    )
    .expect("engine init");
    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 2 },
            display_mode: PriceAxisDisplayMode::Percentage {
                base_price: Some(100.0),
            },
        })
        .expect("set percentage display");

    c.bench_function("price_axis_percentage_display", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_price_axis_log_mode_display(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 0.0, 1_000.0)
            .with_price_domain(1.0, 1_000.0),
    )
    .expect("engine init");
    engine
        .set_price_scale_mode(PriceScaleMode::Log)
        .expect("set log mode");

    c.bench_function("price_axis_log_mode_display", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_price_scale_log_ladder_ticks(c: &mut Criterion) {
    let scale = PriceScale::new_with_mode(1.0, 1_000_000.0, PriceScaleMode::Log).expect("scale");

    c.bench_function("price_scale_log_ladder_ticks_16", |b| {
        b.iter(|| {
            let _ = scale.ticks(16).expect("log ladder ticks");
        })
    });
}

fn bench_price_axis_label_cache_hot(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 0.0, 1_000.0)
            .with_price_domain(95.0, 105.0),
    )
    .expect("engine init");
    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::Adaptive,
            ..PriceAxisLabelConfig::default()
        })
        .expect("set adaptive policy");
    engine.clear_price_label_cache();
    let _ = engine.build_render_frame().expect("warm cache");

    c.bench_function("price_axis_label_cache_hot", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_last_price_marker_render(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 0.0, 2_000.0)
            .with_price_domain(90.0, 140.0),
    )
    .expect("engine init");
    let points: Vec<DataPoint> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            let y = 100.0 + (t * 0.01).sin() * 5.0 + t * 0.01;
            DataPoint::new(t, y)
        })
        .collect();
    engine.set_data(points);
    engine
        .set_render_style(RenderStyle {
            show_last_price_line: true,
            show_last_price_label: true,
            ..engine.render_style()
        })
        .expect("set style");

    c.bench_function("last_price_marker_render", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_last_price_trend_color_render(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(920, 420), 0.0, 2_000.0)
            .with_price_domain(90.0, 140.0),
    )
    .expect("engine init");
    let mut points: Vec<DataPoint> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            let y = 100.0 + (t * 0.01).sin() * 5.0 + t * 0.01;
            DataPoint::new(t, y)
        })
        .collect();
    if points.len() >= 2 {
        let last_index = points.len() - 1;
        points[last_index - 1] = DataPoint::new((last_index - 1) as f64, 119.0);
        points[last_index] = DataPoint::new(last_index as f64, 120.0);
    }
    engine.set_data(points);
    engine
        .set_render_style(RenderStyle {
            show_last_price_line: true,
            show_last_price_label: true,
            last_price_use_trend_color: true,
            ..engine.render_style()
        })
        .expect("set style");

    c.bench_function("last_price_trend_color_render", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

fn bench_last_price_label_collision_filter(c: &mut Criterion) {
    let mut engine = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(320, 240), 0.0, 2_000.0)
            .with_price_domain(90.0, 140.0),
    )
    .expect("engine init");
    let points: Vec<DataPoint> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            let y = 100.0 + (t * 0.01).sin() * 5.0 + t * 0.01;
            DataPoint::new(t, y)
        })
        .collect();
    engine.set_data(points);
    engine
        .set_render_style(RenderStyle {
            show_last_price_line: false,
            show_last_price_label: true,
            last_price_label_exclusion_px: 10_000.0,
            ..engine.render_style()
        })
        .expect("set style");

    c.bench_function("last_price_label_collision_filter", |b| {
        b.iter(|| {
            let _ = engine.build_render_frame().expect("build render frame");
        })
    });
}

criterion_group!(
    benches,
    bench_linear_scale_round_trip,
    bench_candle_projection_10k,
    bench_bar_projection_10k,
    bench_line_projection_20k,
    bench_area_projection_20k,
    bench_baseline_projection_20k,
    bench_histogram_projection_20k,
    bench_visible_window_points_100k,
    bench_marker_placement_5k,
    bench_plugin_dispatch_pointer_move,
    bench_crosshair_modes_pointer_move,
    bench_wheel_zoom_step,
    bench_wheel_pan_step,
    bench_kinetic_pan_step,
    bench_render_frame_build_20k,
    bench_render_axis_layout_narrow,
    bench_time_axis_datetime_formatter,
    bench_time_axis_label_cache_hot,
    bench_time_axis_session_timezone_formatter,
    bench_render_major_time_tick_styling,
    bench_price_axis_min_move_formatter,
    bench_price_axis_percentage_display,
    bench_price_axis_log_mode_display,
    bench_price_scale_log_ladder_ticks,
    bench_price_axis_label_cache_hot,
    bench_last_price_marker_render,
    bench_last_price_trend_color_render,
    bench_last_price_label_collision_filter,
    bench_engine_snapshot_json_2k
);
criterion_main!(benches);
