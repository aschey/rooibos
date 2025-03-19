use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ratatui::layout::Rect;
use terminput::ScrollDirection;
use wasm_compat::cell::BoolCell;

use super::{
    BlurEvent, ClickEventProps, DragEventProps, EventData, EventHandle, FocusEvent, KeyEventProps,
    NodeState,
};

pub trait IntoKeyHandler {
    fn into_key_handler(self) -> impl KeyHandler;
}

impl<T> IntoKeyHandler for T
where
    T: KeyHandler,
{
    fn into_key_handler(self) -> impl KeyHandler {
        self
    }
}

pub trait KeyHandler {
    fn handle(&mut self, props: KeyEventProps);
}

impl<F> KeyHandler for F
where
    F: FnMut(KeyEventProps),
{
    fn handle(&mut self, props: KeyEventProps) {
        self(props)
    }
}

impl KeyHandler for Box<dyn KeyHandler> {
    fn handle(&mut self, props: KeyEventProps) {
        (**self).handle(props)
    }
}

pub trait IntoClickHandler {
    fn into_click_handler(self) -> impl ClickHandler;
}

impl<T> IntoClickHandler for T
where
    T: ClickHandler,
{
    fn into_click_handler(self) -> impl ClickHandler {
        self
    }
}

pub trait ClickHandler {
    fn handle(&mut self, props: ClickEventProps);
}

impl<F> ClickHandler for F
where
    F: FnMut(ClickEventProps),
{
    fn handle(&mut self, props: ClickEventProps) {
        self(props)
    }
}

impl ClickHandler for Box<dyn ClickHandler> {
    fn handle(&mut self, props: ClickEventProps) {
        (**self).handle(props)
    }
}

pub trait IntoDragHandler {
    fn into_drag_handler(self) -> impl DragHandler;
}

impl<T> IntoDragHandler for T
where
    T: DragHandler,
{
    fn into_drag_handler(self) -> impl DragHandler {
        self
    }
}

pub trait DragHandler {
    fn handle(&mut self, props: DragEventProps);
}

impl<F> DragHandler for F
where
    F: FnMut(DragEventProps),
{
    fn handle(&mut self, props: DragEventProps) {
        self(props)
    }
}

impl DragHandler for Box<dyn DragHandler> {
    fn handle(&mut self, props: DragEventProps) {
        (**self).handle(props)
    }
}

pub(crate) type KeyEventFn = Rc<RefCell<dyn KeyHandler>>;
pub(crate) type ClickEventFn = Rc<RefCell<dyn ClickHandler>>;
pub(crate) type DragEventFn = Rc<RefCell<dyn DragHandler>>;
pub(crate) type EventFn = Rc<RefCell<dyn FnMut(EventData, EventHandle)>>;
pub(crate) type SizeChangeFn = Rc<RefCell<dyn FnMut(Rect)>>;
pub(crate) type PasteFn = Rc<RefCell<dyn FnMut(String, EventData, EventHandle)>>;
pub(crate) type FocusFn = Rc<RefCell<dyn FnMut(FocusEvent, EventData)>>;
pub(crate) type BlurFn = Rc<RefCell<dyn FnMut(BlurEvent, EventData)>>;
pub(crate) type ScrollFn = Rc<RefCell<dyn FnMut(ScrollDirection, EventData, EventHandle)>>;

#[derive(Clone, Default)]
pub struct EventHandlers {
    pub(crate) on_key_down: Vec<KeyEventFn>,
    pub(crate) on_key_up: Vec<KeyEventFn>,
    pub(crate) on_paste: Vec<PasteFn>,
    pub(crate) on_focus: Vec<FocusFn>,
    pub(crate) on_blur: Vec<BlurFn>,
    pub(crate) on_click: Vec<ClickEventFn>,
    pub(crate) on_right_click: Vec<ClickEventFn>,
    pub(crate) on_middle_click: Vec<ClickEventFn>,
    pub(crate) on_mouse_enter: Vec<EventFn>,
    pub(crate) on_mouse_leave: Vec<EventFn>,
    pub(crate) on_mouse_drag: Vec<DragEventFn>,
    pub(crate) on_size_change: Vec<SizeChangeFn>,
    pub(crate) on_scroll: Vec<ScrollFn>,
    pub(crate) on_enable: Vec<EventFn>,
    pub(crate) on_disable: Vec<EventFn>,
}

impl EventHandlers {
    pub fn on_key_down<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        self.on_key_down
            .push(Rc::new(RefCell::new(handler.into_key_handler())));
        self
    }

    pub fn on_key_up<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        self.on_key_up
            .push(Rc::new(RefCell::new(handler.into_key_handler())));
        self
    }

    pub fn on_paste<F>(mut self, handler: F) -> Self
    where
        F: FnMut(String, EventData, EventHandle) + 'static,
    {
        self.on_paste.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_focus<F>(mut self, handler: F) -> Self
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        self.on_focus.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_blur<F>(mut self, handler: F) -> Self
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        self.on_blur.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_state_change<F>(self, handler: F) -> Self
    where
        F: FnMut(NodeState, EventData) + 'static,
    {
        let handler = Rc::new(RefCell::new(handler));
        let focused = Arc::new(BoolCell::new(false));
        let hovered = Arc::new(BoolCell::new(false));
        let enabled = Arc::new(BoolCell::new(true));
        let get_state = {
            let focused = focused.clone();
            let hovered = hovered.clone();
            let enabled = enabled.clone();
            move || {
                let mut state = NodeState::empty();
                if focused.get() {
                    state |= NodeState::FOCUSED;
                }
                if hovered.get() {
                    state |= NodeState::HOVERED;
                }
                if !enabled.get() {
                    state |= NodeState::DISABLED;
                }
                state
            }
        };
        self.on_focus({
            let get_state = get_state.clone();
            let focused = focused.clone();
            let handler = handler.clone();
            move |_e, data| {
                focused.set(true);
                handler.borrow_mut()(get_state(), data);
            }
        })
        .on_blur({
            let get_state = get_state.clone();
            let focused = focused.clone();
            let handler = handler.clone();
            move |_e, data| {
                focused.set(false);
                handler.borrow_mut()(get_state(), data);
            }
        })
        .on_mouse_enter({
            let get_state = get_state.clone();
            let hovered = hovered.clone();
            let handler = handler.clone();
            move |data, _handle| {
                hovered.set(true);
                handler.borrow_mut()(get_state(), data);
            }
        })
        .on_mouse_leave({
            let get_state = get_state.clone();
            let handler = handler.clone();
            move |data, _handle| {
                hovered.set(false);
                handler.borrow_mut()(get_state(), data);
            }
        })
        .on_enable({
            let get_state = get_state.clone();
            let handler = handler.clone();
            let enabled = enabled.clone();
            move |data, _handle| {
                enabled.set(true);
                handler.borrow_mut()(get_state(), data);
            }
        })
        .on_disable({
            move |data, _handle| {
                enabled.set(false);
                handler.borrow_mut()(get_state(), data);
            }
        })
    }

    pub fn on_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.on_click
            .push(Rc::new(RefCell::new(handler.into_click_handler())));
        self
    }

    pub fn on_right_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.on_right_click
            .push(Rc::new(RefCell::new(handler.into_click_handler())));
        self
    }

    pub fn on_middle_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.on_middle_click
            .push(Rc::new(RefCell::new(handler.into_click_handler())));
        self
    }

    pub fn on_mouse_drag<H>(mut self, handler: H) -> Self
    where
        H: IntoDragHandler + 'static,
    {
        self.on_mouse_drag
            .push(Rc::new(RefCell::new(handler.into_drag_handler())));
        self
    }

    pub fn on_mouse_enter<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.on_mouse_enter.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_mouse_leave<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.on_mouse_leave.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_size_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(Rect) + 'static,
    {
        self.on_size_change.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_scroll<F>(mut self, handler: F) -> Self
    where
        F: FnMut(ScrollDirection, EventData, EventHandle) + 'static,
    {
        self.on_scroll.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_enable<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.on_enable.push(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_disable<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.on_disable.push(Rc::new(RefCell::new(handler)));
        self
    }
}
