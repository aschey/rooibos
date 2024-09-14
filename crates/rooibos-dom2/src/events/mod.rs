pub use dispatcher::*;
pub(crate) use event_handler::*;
use ratatui::layout::Rect;

use crate::NodeId;

mod dispatcher;
mod event_handler;

#[derive(Debug, Default)]
pub struct EventHandle {
    stop_propagation: bool,
}

impl EventHandle {
    pub fn stop_propagation(&mut self) {
        self.stop_propagation = true;
    }
}

#[derive(Debug)]
pub struct EventData {
    pub rect: Rect,
}

#[derive(Debug)]
pub struct BlurEvent {
    pub new_target: Option<NodeId>,
}

#[derive(Debug)]
pub struct FocusEvent {
    pub previous_target: Option<NodeId>,
}
