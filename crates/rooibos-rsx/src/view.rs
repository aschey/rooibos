use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::Backend;

pub trait View<B: Backend>: 'static {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect);
    fn into_boxed_view(self) -> Box<dyn View<B>>;
}

impl<B, F> View<B> for F
where
    B: Backend,
    F: FnMut(&mut Frame<B>, Rect) + 'static,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self)(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

impl<B> View<B> for Box<dyn View<B>>
where
    B: Backend,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (**self).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        self
    }
}

impl<B> View<B> for Vec<Box<dyn View<B>>>
where
    B: Backend,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        let size = rect.height / self.len() as u16;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(size); self.len()])
            .split(rect);
        for (i, item) in self.iter_mut().enumerate() {
            item.view(frame, chunks[i]);
        }
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

impl<B> View<B> for Rc<RefCell<dyn View<B>>>
where
    B: Backend,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        self.borrow_mut().view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

impl<B, V> View<B> for Rc<RefCell<V>>
where
    B: Backend,
    V: View<B>,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        self.borrow_mut().view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

pub trait LazyView<B: Backend>: 'static {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect);
}

impl<B, F, Ret> LazyView<B> for F
where
    B: Backend,
    F: FnMut() -> Ret + 'static,
    Ret: View<B>,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self)().view(frame, rect)
    }
}

pub struct LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    f: F,
    _phantom: PhantomData<B>,
}

impl<B, F> LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<B, F> View<B> for LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self.f).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
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

pub trait IntoBoxedLazyView<B>
where
    B: Backend,
{
    fn into_boxed_lazy_view(self) -> Box<dyn Fn() -> Box<dyn View<B>>>;
}

impl<B, F, V> IntoBoxedLazyView<B> for F
where
    B: Backend,
    F: Fn() -> V + 'static,
    V: View<B>,
{
    fn into_boxed_lazy_view(self) -> Box<dyn Fn() -> Box<dyn View<B>>> {
        (move || (self)().into_boxed_view()).into_boxed()
    }
}
