use std::cell::RefCell;
use std::rc::Rc;

use crate::KeyEvent;

pub(crate) type KeyEventFn = Rc<RefCell<dyn FnMut(KeyEvent)>>;

pub(crate) type EventFn = Rc<RefCell<dyn FnMut()>>;

#[derive(Clone, Default)]
pub(crate) struct EventHandlers {
    pub(crate) on_key_down: Option<KeyEventFn>,
    pub(crate) on_key_up: Option<KeyEventFn>,
    pub(crate) on_focus: Option<EventFn>,
    pub(crate) on_blur: Option<EventFn>,
}

impl EventHandlers {
    pub(crate) fn on_key_down<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent) + 'static,
    {
        self.on_key_down = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_key_up<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent) + 'static,
    {
        self.on_key_up = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_focus<F>(mut self, handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.on_focus = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_blur<F>(mut self, handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.on_blur = Some(Rc::new(RefCell::new(handler)));
        self
    }
}
