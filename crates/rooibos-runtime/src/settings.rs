use std::time::Duration;

#[derive(Debug)]
pub struct RuntimeSettings {
    pub(crate) enable_input_reader: bool,
    pub(crate) enable_signal_handler: bool,
    pub(crate) show_final_output: bool,
    pub(crate) hover_debounce: Duration,
    pub(crate) resize_debounce: Duration,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            enable_input_reader: true,
            enable_signal_handler: true,
            show_final_output: true,
            hover_debounce: Duration::from_millis(20),
            resize_debounce: Duration::from_millis(20),
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
        self.show_final_output = show_final_output;
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
}
