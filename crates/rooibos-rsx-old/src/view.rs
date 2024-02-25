use std::cell::RefCell;
use std::rc::Rc;

use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

pub trait View: 'static {
    fn view(&mut self, frame: &mut Frame, rect: Rect);
    fn into_boxed_view(self) -> Box<dyn View>;
}

impl<F> View for F
where
    F: FnMut(&mut Frame, Rect) + 'static,
{
    fn view(&mut self, frame: &mut Frame, rect: Rect) {
        (self)(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View> {
        Box::new(self)
    }
}

impl View for Box<dyn View> {
    fn view(&mut self, frame: &mut Frame, rect: Rect) {
        (**self).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View> {
        self
    }
}

impl View for Vec<Box<dyn View>> {
    fn view(&mut self, frame: &mut Frame, rect: Rect) {
        let size = rect.height / self.len() as u16;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(size); self.len()])
            .split(rect);
        for (i, item) in self.iter_mut().enumerate() {
            item.view(frame, chunks[i]);
        }
    }

    fn into_boxed_view(self) -> Box<dyn View> {
        Box::new(self)
    }
}

impl View for Rc<RefCell<dyn View>> {
    fn view(&mut self, frame: &mut Frame, rect: Rect) {
        self.borrow_mut().view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View> {
        Box::new(self)
    }
}

impl<V> View for Rc<RefCell<V>>
where
    V: View,
{
    fn view(&mut self, frame: &mut Frame, rect: Rect) {
        self.borrow_mut().view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View> {
        Box::new(self)
    }
}

pub trait LazyView: 'static {
    fn view(&mut self, frame: &mut Frame, rect: Rect);
}

impl<F, Ret> LazyView for F
where
    F: FnMut() -> Ret + 'static,
    Ret: View,
{
    fn view(&mut self, frame: &mut Frame, rect: Rect) {
        (self)().view(frame, rect)
    }
}

pub struct LazyViewWrapper<F>
where
    F: LazyView,
{
    f: F,
}

impl<F> LazyViewWrapper<F>
where
    F: LazyView,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> View for LazyViewWrapper<F>
where
    F: LazyView,
{
    fn view(&mut self, frame: &mut Frame, rect: Rect) {
        (self.f).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View> {
        Box::new(self)
    }
}

pub trait IntoBoxed<T: ?Sized> {
    fn into_boxed(self) -> Box<T>;
}

impl<F, R> IntoBoxed<dyn Fn() -> R> for F
where
    F: Fn() -> R + 'static,
{
    fn into_boxed(self: F) -> Box<dyn Fn() -> R> {
        Box::new(self)
    }
}

pub trait IntoBoxedViewFn {
    fn into_boxed_view_fn(self) -> Box<dyn Fn() -> Box<dyn View>>;
}

impl<F, V> IntoBoxedViewFn for F
where
    F: Fn() -> V + 'static,
    V: View,
{
    fn into_boxed_view_fn(self) -> Box<dyn Fn() -> Box<dyn View>> {
        (move || (self)().into_boxed_view()).into_boxed()
    }
}
