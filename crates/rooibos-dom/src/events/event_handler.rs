use std::cell::RefCell;
use std::rc::Rc;

use crate::{EventData, KeyEvent, MouseEvent};

pub(crate) type KeyEventFn = Rc<RefCell<dyn FnMut(KeyEvent, EventData)>>;

pub(crate) type MouseEventFn = Rc<RefCell<dyn FnMut(MouseEvent, EventData)>>;

pub(crate) type EventFn = Rc<RefCell<dyn FnMut(EventData)>>;

#[derive(Clone, Default)]
pub(crate) struct EventHandlers {
    pub(crate) on_key_down: Option<KeyEventFn>,
    pub(crate) on_key_up: Option<KeyEventFn>,
    pub(crate) on_focus: Option<EventFn>,
    pub(crate) on_blur: Option<EventFn>,
    pub(crate) on_click: Option<MouseEventFn>,
    pub(crate) on_mouse_enter: Option<EventFn>,
    pub(crate) on_mouse_leave: Option<EventFn>,
}

impl EventHandlers {
    pub(crate) fn on_key_down<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        self.on_key_down = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_key_up<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        self.on_key_up = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_focus<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.on_focus = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_blur<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.on_blur = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_click<F>(mut self, handler: F) -> Self
    where
        F: FnMut(MouseEvent, EventData) + 'static,
    {
        self.on_click = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_mouse_enter<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.on_mouse_enter = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_mouse_leave<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.on_mouse_leave = Some(Rc::new(RefCell::new(handler)));
        self
    }
}
