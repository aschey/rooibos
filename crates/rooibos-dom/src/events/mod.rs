mod event_handler;

pub(crate) use event_handler::*;
use ratatui::layout::Rect;

pub struct EventData {
    pub rect: Rect,
}
