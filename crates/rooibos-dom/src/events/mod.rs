use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bitflags::bitflags;
pub use dispatcher::*;
pub use event_handler::*;
use ratatui::layout::Rect;
use terminput::{KeyEvent, KeyModifiers, MouseButton};

use crate::{DomNodeKey, NodeId};

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

pub enum Event {
    Key(terminput::KeyEvent),
    Mouse(terminput::MouseEvent),
    WindowFocusGained,
    WindowFocusLost,
    Paste(String),
    Resize,
    NodeEnable(DomNodeKey),
    NodeDisable(DomNodeKey),
    NodeBlur {
        blur_key: DomNodeKey,
        focus_target: Option<NodeId>,
    },
    NodeFocus {
        focus_key: DomNodeKey,
        prev_focused: Option<NodeId>,
    },
}

impl From<terminput::Event> for Event {
    fn from(value: terminput::Event) -> Self {
        match value {
            terminput::Event::FocusGained => Event::WindowFocusGained,
            terminput::Event::FocusLost => Event::WindowFocusLost,
            terminput::Event::Key(key_event) => Event::Key(key_event),
            terminput::Event::Mouse(mouse_event) => Event::Mouse(mouse_event),
            terminput::Event::Paste(text) => Event::Paste(text),
            terminput::Event::Resize { .. } => Event::Resize,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub rect: Rect,
    pub target: Option<NodeId>,
    pub is_direct: bool,
}

#[derive(Debug)]
pub struct BlurEvent {
    pub new_target: Option<NodeId>,
}

#[derive(Debug)]
pub struct FocusEvent {
    pub previous_target: Option<NodeId>,
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct NodeState: u32 {
        const FOCUSED = 0b001;
        const HOVERED = 0b010;
        const DISABLED = 0b100;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StateChangeCause {
    Focus,
    Blur,
    MouseEnter,
    MouseLeave,
    Enable,
    Disable,
}

#[derive(Clone, Copy)]
pub struct StateChangeEvent {
    pub state: NodeState,
    pub cause: StateChangeCause,
}

#[derive(Clone, Debug)]
pub struct ClickEvent {
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

pub struct DragEvent {
    pub button: MouseButton,
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

pub struct DragEventProps {
    pub event: DragEvent,
    pub data: EventData,
    pub handle: EventHandle,
}
