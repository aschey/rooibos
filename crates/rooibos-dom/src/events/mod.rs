use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use dispatcher::*;
pub use event_handler::*;
use ratatui::layout::Rect;
use terminput::{KeyEvent, KeyModifiers};

use crate::NodeId;

mod dispatcher;
mod event_handler;

#[derive(Debug, Clone, Default)]
pub struct EventHandle {
    stop_propagation: Arc<AtomicBool>,
}

impl EventHandle {
    pub fn stop_propagation(&mut self) {
        self.stop_propagation.store(true, Ordering::Relaxed)
    }

    pub(crate) fn get_stop_propagation(&self) -> bool {
        self.stop_propagation.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Clone, Debug)]
pub struct ClickEvent {
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

#[derive(Clone, Debug)]
pub struct KeyEventProps {
    pub event: KeyEvent,
    pub data: EventData,
    pub handle: EventHandle,
}

#[derive(Clone, Debug)]
pub struct ClickEventProps {
    pub event: ClickEvent,
    pub data: EventData,
    pub handle: EventHandle,
}
