pub use dispatcher::*;
pub(crate) use event_handler::*;
use ratatui::layout::Rect;

use crate::NodeId;

mod dispatcher;
mod event_handler;

pub struct EventData {
    pub rect: Rect,
}

pub struct BlurEvent {
    pub new_target: Option<NodeId>,
}

pub struct FocusEvent {
    pub previous_target: Option<NodeId>,
}
