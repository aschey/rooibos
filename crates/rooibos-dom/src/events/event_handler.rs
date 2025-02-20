use std::cell::RefCell;
use std::rc::Rc;

use ratatui::layout::Rect;
use terminput::ScrollDirection;

use super::{
    BlurEvent, ClickEventProps, DragEventProps, EventData, EventHandle, FocusEvent, KeyEventProps,
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
    pub(crate) on_key_down: Option<KeyEventFn>,
    pub(crate) on_key_up: Option<KeyEventFn>,
    pub(crate) on_paste: Option<PasteFn>,
    pub(crate) on_focus: Option<FocusFn>,
    pub(crate) on_blur: Option<BlurFn>,
    pub(crate) on_click: Option<ClickEventFn>,
    pub(crate) on_right_click: Option<ClickEventFn>,
    pub(crate) on_middle_click: Option<ClickEventFn>,
    pub(crate) on_mouse_enter: Option<EventFn>,
    pub(crate) on_mouse_leave: Option<EventFn>,
    pub(crate) on_mouse_drag: Option<DragEventFn>,
    pub(crate) on_size_change: Option<SizeChangeFn>,
    pub(crate) on_scroll: Option<ScrollFn>,
}

impl EventHandlers {
    pub fn on_key_down<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        self.on_key_down = Some(Rc::new(RefCell::new(handler.into_key_handler())));
        self
    }

    pub fn on_key_up<H>(mut self, handler: H) -> Self
    where
        H: IntoKeyHandler + 'static,
    {
        self.on_key_up = Some(Rc::new(RefCell::new(handler.into_key_handler())));
        self
    }

    pub fn on_paste<F>(mut self, handler: F) -> Self
    where
        F: FnMut(String, EventData, EventHandle) + 'static,
    {
        self.on_paste = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_focus<F>(mut self, handler: F) -> Self
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        self.on_focus = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_blur<F>(mut self, handler: F) -> Self
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        self.on_blur = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.on_click = Some(Rc::new(RefCell::new(handler.into_click_handler())));
        self
    }

    pub fn on_right_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.on_right_click = Some(Rc::new(RefCell::new(handler.into_click_handler())));
        self
    }

    pub fn on_middle_click<H>(mut self, handler: H) -> Self
    where
        H: IntoClickHandler + 'static,
    {
        self.on_middle_click = Some(Rc::new(RefCell::new(handler.into_click_handler())));
        self
    }

    pub fn on_mouse_drag<H>(mut self, handler: H) -> Self
    where
        H: IntoDragHandler + 'static,
    {
        self.on_mouse_drag = Some(Rc::new(RefCell::new(handler.into_drag_handler())));
        self
    }

    pub fn on_mouse_enter<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.on_mouse_enter = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_mouse_leave<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData, EventHandle) + 'static,
    {
        self.on_mouse_leave = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_size_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(Rect) + 'static,
    {
        self.on_size_change = Some(Rc::new(RefCell::new(handler)));
        self
    }

    pub fn on_scroll<F>(mut self, handler: F) -> Self
    where
        F: FnMut(ScrollDirection, EventData, EventHandle) + 'static,
    {
        self.on_scroll = Some(Rc::new(RefCell::new(handler)));
        self
    }
}
