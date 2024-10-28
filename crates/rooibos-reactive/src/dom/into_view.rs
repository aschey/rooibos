use tachys::view::Render;

use crate::dom::{RenderAny, RooibosDom};

pub struct View<T>(T)
where
    T: Sized;

impl<T> View<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub trait IntoView: Sized + RenderAny {
    fn into_view(self) -> View<Self>;
}

impl<T> IntoView for T
where
    T: Sized + RenderAny,
{
    fn into_view(self) -> View<Self> {
        View(self)
    }
}

impl<T: RenderAny> Render<RooibosDom> for View<T> {
    type State = T::State;

    fn build(self) -> Self::State {
        self.0.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.0.rebuild(state)
    }
}
