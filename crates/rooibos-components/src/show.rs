use either_of::Either;
use rooibos_reactive::dom::{IntoView, TypedChildrenMut, ViewFn};
use rooibos_reactive::graph::traits::Get;

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

    pub fn render<C, W>(self, when: W, children: impl Into<TypedChildrenMut<C>>) -> impl IntoView
    where
        C: IntoView + 'static,
        W: Get<Value = bool> + Send + 'static,
    {
        let Self { fallback } = self;

        let mut children = children.into().into_inner();
        move || {
            if when.get() {
                Either::Left(children())
            } else {
                Either::Right(fallback.run())
            }
        }
    }
}
