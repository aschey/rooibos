use std::cell::RefCell;
use std::rc::Rc;

use ratatui::layout::Rect;
use terminput::{KeyEvent, MouseEvent};

use super::{BlurEvent, FocusEvent};
use crate::{ClickEvent, EventData, EventHandle};

pub(crate) type KeyEventFn = Rc<RefCell<dyn FnMut(KeyEvent, EventData, &mut EventHandle)>>;
pub(crate) type ClickEventFn = Rc<RefCell<dyn FnMut(ClickEvent, EventData, &mut EventHandle)>>;
pub(crate) type EventFn = Rc<RefCell<dyn FnMut(EventData, &mut EventHandle)>>;
pub(crate) type SizeChangeFn = Rc<RefCell<dyn FnMut(Rect)>>;
pub(crate) type PasteFn = Rc<RefCell<dyn FnMut(String, EventData, &mut EventHandle)>>;
pub(crate) type FocusFn = Rc<RefCell<dyn FnMut(FocusEvent, EventData)>>;
pub(crate) type BlurFn = Rc<RefCell<dyn FnMut(BlurEvent, EventData)>>;

#[derive(Clone, Default)]
pub(crate) struct EventHandlers {
    pub(crate) on_key_down: Option<KeyEventFn>,
    pub(crate) on_key_up: Option<KeyEventFn>,
    pub(crate) on_paste: Option<PasteFn>,
    pub(crate) on_focus: Option<FocusFn>,
    pub(crate) on_blur: Option<BlurFn>,
    pub(crate) on_click: Option<ClickEventFn>,
    pub(crate) on_mouse_enter: Option<EventFn>,
    pub(crate) on_mouse_leave: Option<EventFn>,
    pub(crate) on_size_change: Option<SizeChangeFn>,
}

impl EventHandlers {
    pub(crate) fn on_key_down<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData, &mut EventHandle) + 'static,
    {
        self.on_key_down = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_key_up<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData, &mut EventHandle) + 'static,
    {
        self.on_key_up = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_paste<F>(mut self, handler: F) -> Self
    where
        F: FnMut(String, EventData, &mut EventHandle) + 'static,
    {
        self.on_paste = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_focus<F>(mut self, handler: F) -> Self
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        self.on_focus = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_blur<F>(mut self, handler: F) -> Self
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        self.on_blur = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_click<F>(mut self, handler: F) -> Self
    where
        F: FnMut(ClickEvent, EventData, &mut EventHandle) + 'static,
    {
        self.on_click = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_mouse_enter<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, &mut EventHandle) + 'static,
    {
        self.on_mouse_enter = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_mouse_leave<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, &mut EventHandle) + 'static,
    {
        self.on_mouse_leave = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub(crate) fn on_size_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(Rect) + 'static,
    {
        self.on_size_change = Some(Rc::new(RefCell::new(handler)));
        self
    }
}
