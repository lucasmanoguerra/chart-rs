use std::cell::RefCell;
use std::rc::Rc;

use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::extensions::{ChartPlugin, PluginContext, PluginEvent};
use chart_rs::render::NullRenderer;

#[derive(Clone)]
struct RecordingPlugin {
    id: String,
    events: Rc<RefCell<Vec<PluginEvent>>>,
}

impl RecordingPlugin {
    fn new(id: impl Into<String>, events: Rc<RefCell<Vec<PluginEvent>>>) -> Self {
        Self {
            id: id.into(),
            events,
        }
    }
}

impl ChartPlugin for RecordingPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_event(&mut self, event: PluginEvent, _context: PluginContext) {
        self.events.borrow_mut().push(event);
    }
}

fn event_kind(event: &PluginEvent) -> &'static str {
    match event {
        PluginEvent::DataUpdated { .. } => "data",
        PluginEvent::CandlesUpdated { .. } => "candles",
        PluginEvent::PointerMoved { .. } => "pointer_move",
        PluginEvent::PointerLeft => "pointer_leave",
        PluginEvent::VisibleRangeChanged { .. } => "range",
        PluginEvent::PanStarted => "pan_start",
        PluginEvent::PanEnded => "pan_end",
        PluginEvent::Rendered => "rendered",
    }
}

#[test]
fn plugin_receives_deterministic_event_sequence() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let events = Rc::new(RefCell::new(Vec::<PluginEvent>::new()));
    engine
        .register_plugin(Box::new(RecordingPlugin::new("recorder", events.clone())))
        .expect("register plugin");

    engine.set_data(vec![DataPoint::new(10.0, 10.0)]);
    engine.append_point(DataPoint::new(20.0, 20.0));
    engine.set_candles(vec![
        OhlcBar::new(10.0, 9.0, 12.0, 8.0, 11.0).expect("valid candle"),
    ]);
    engine.pointer_move(100.0, 200.0);
    engine
        .set_time_visible_range(10.0, 50.0)
        .expect("set visible range");
    engine.pan_start();
    engine.pan_end();
    engine.render().expect("render");
    engine.pointer_leave();

    let events = events.borrow();
    let kinds: Vec<&'static str> = events.iter().map(event_kind).collect();
    assert_eq!(
        kinds,
        vec![
            "data",
            "data",
            "candles",
            "pointer_move",
            "range",
            "pan_start",
            "pan_end",
            "rendered",
            "pointer_leave",
        ]
    );
}

#[test]
fn duplicate_plugin_ids_are_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let events = Rc::new(RefCell::new(Vec::<PluginEvent>::new()));
    engine
        .register_plugin(Box::new(RecordingPlugin::new("dupe", events.clone())))
        .expect("first plugin");
    let err = engine
        .register_plugin(Box::new(RecordingPlugin::new("dupe", events)))
        .expect_err("duplicate must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn unregister_plugin_stops_dispatch() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let events = Rc::new(RefCell::new(Vec::<PluginEvent>::new()));
    engine
        .register_plugin(Box::new(RecordingPlugin::new("to-remove", events.clone())))
        .expect("register");
    assert_eq!(engine.plugin_count(), 1);
    assert!(engine.has_plugin("to-remove"));

    engine.set_data(vec![DataPoint::new(1.0, 1.0)]);
    assert!(engine.unregister_plugin("to-remove"));
    assert_eq!(engine.plugin_count(), 0);
    assert!(!engine.has_plugin("to-remove"));

    engine.append_point(DataPoint::new(2.0, 2.0));
    assert_eq!(events.borrow().len(), 1);
}

#[test]
fn visible_range_event_contains_new_range() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let events = Rc::new(RefCell::new(Vec::<PluginEvent>::new()));
    engine
        .register_plugin(Box::new(RecordingPlugin::new("range", events.clone())))
        .expect("register");

    engine
        .set_time_visible_range(15.0, 40.0)
        .expect("set visible range");
    let last = events
        .borrow()
        .last()
        .copied()
        .expect("range event expected");
    match last {
        PluginEvent::VisibleRangeChanged { start, end } => {
            assert!((start - 15.0).abs() <= 1e-9);
            assert!((end - 40.0).abs() <= 1e-9);
        }
        _ => panic!("expected visible range event"),
    }
}
