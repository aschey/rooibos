use tachys::view::Render;

use crate::RooibosDom;

pub struct View<T>(T)
where
    T: Sized;

impl<T> View<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub trait IntoView: Sized + Render<RooibosDom> {
    fn into_view(self) -> View<Self>;
}

impl<T> IntoView for T
where
    T: Sized + Render<RooibosDom>,
{
    fn into_view(self) -> View<Self> {
        View(self)
    }
}

impl<T: Render<RooibosDom>> Render<RooibosDom> for View<T> {
    type State = T::State;
    type FallibleState = T::FallibleState;

    fn build(self) -> Self::State {
        self.0.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.0.rebuild(state)
    }

    fn try_build(self) -> any_error::Result<Self::FallibleState> {
        self.0.try_build()
    }

    fn try_rebuild(self, state: &mut Self::FallibleState) -> any_error::Result<()> {
        self.0.try_rebuild(state)
    }
}
