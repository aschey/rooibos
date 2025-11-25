use either_of::Either;
use rooibos_reactive::dom::{IntoView, TypedChildrenMut, ViewFn};
use rooibos_reactive::graph::IntoReactiveValue;
use rooibos_reactive::graph::traits::Get;
use rooibos_reactive::graph::wrappers::read::Signal;

pub struct Show {
    fallback: ViewFn,
}

impl Default for Show {
    fn default() -> Self {
        Self::new()
    }
}

impl Show {
    pub fn new() -> Self {
        Self {
            fallback: (|| {}).into(),
        }
    }

    pub fn fallback(mut self, fallback: impl Into<ViewFn>) -> Self {
        self.fallback = fallback.into();
        self
    }

    pub fn render<C, W, M>(self, when: W, children: impl Into<TypedChildrenMut<C>>) -> impl IntoView
    where
        C: IntoView + 'static,
        W: IntoReactiveValue<Signal<bool>, M>,
    {
        let Self { fallback } = self;

        let mut children = children.into().into_inner();
        let when = when.into_reactive_value();
        move || {
            if when.get() {
                Either::Left(children())
            } else {
                Either::Right(fallback.run())
            }
        }
    }
}
