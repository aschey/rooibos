use std::sync::Arc;
use std::time::Duration;

use educe::Educe;
use ratatui::Viewport;
use rooibos_dom::{Event, KeyCode, KeyEvent, KeyModifiers};
use wasm_compat::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputMode {
    Normal,
    Insert,
}

pub type IsQuitEvent = dyn Fn(KeyEvent) -> bool + Send + Sync;
pub type EventFilter = dyn FnMut(Event, InputMode) -> Option<Event> + Send + Sync;

#[derive(Educe)]
#[educe(Debug)]
pub struct RuntimeSettings {
    pub(crate) enable_input_reader: bool,
    pub(crate) enable_signal_handler: bool,
    pub(crate) show_final_output: Option<bool>,
    pub(crate) hover_debounce: Duration,
    pub(crate) resize_debounce: Duration,
    pub(crate) viewport: Viewport,
    #[educe(Debug(ignore))]
    pub(crate) is_quit_event: Arc<IsQuitEvent>,
    #[educe(Debug(ignore))]
    pub(crate) event_filter: Arc<Mutex<Box<EventFilter>>>,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            enable_input_reader: true,
            enable_signal_handler: true,
            show_final_output: None,
            viewport: Viewport::Fullscreen,
            hover_debounce: Duration::from_millis(20),
            resize_debounce: Duration::from_millis(20),
            is_quit_event: Arc::new(|key| {
                let ctrl_c = key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CTRL;
                let q = key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::empty();
                ctrl_c || q
            }),
            event_filter: Arc::new(Mutex::new(Box::new(|event, _| Some(event)))),
        }
    }
}

impl RuntimeSettings {
    pub fn enable_input_reader(mut self, enable: bool) -> Self {
        self.enable_input_reader = enable;
        self
    }

    pub fn enable_signal_handler(mut self, enable: bool) -> Self {
        self.enable_signal_handler = enable;
        self
    }

    pub fn show_final_output(mut self, show_final_output: bool) -> Self {
        self.show_final_output = Some(show_final_output);
        self
    }

    pub fn hover_debounce(mut self, hover_debounce: Duration) -> Self {
        self.hover_debounce = hover_debounce;
        self
    }

    pub fn resize_debounce(mut self, resize_debounce: Duration) -> Self {
        self.resize_debounce = resize_debounce;
        self
    }

    pub fn viewport(mut self, viewport: Viewport) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn is_quit_event<F>(mut self, f: F) -> Self
    where
        F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
    {
        self.is_quit_event = Arc::new(f);
        self
    }

    pub fn event_filter<F>(mut self, f: F) -> Self
    where
        F: FnMut(Event, InputMode) -> Option<Event> + Send + Sync + 'static,
    {
        self.event_filter = Arc::new(Mutex::new(Box::new(f)));
        self
    }
}
