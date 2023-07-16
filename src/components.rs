use core::marker::PhantomData;
use leptos_reactive::{create_memo, Scope, SignalGet};
use ratatui::backend::Backend;
use tui_rsx::{prelude::*, view};
use typed_builder::TypedBuilder;

#[component]
pub fn Show<B, F1, F2, V1, V2, W>(
    _cx: Scope,
    #[prop(children)] children: F1,
    when: W,

    fallback: F2,
) -> impl View<B>
where
    B: Backend + 'static,
    W: Fn() -> bool + 'static,
    F1: Fn() -> V1 + 'static,
    F2: Fn() -> V2 + 'static,
    V1: View<B> + 'static,
    V2: View<B> + 'static,
{
    move || match when() {
        true => children().into_boxed_view(),
        false => fallback().into_boxed_view(),
    }
}

#[derive(TypedBuilder)]
pub struct Case<B>
where
    B: Backend + 'static,
{
    #[builder(setter(!strip_option, transform = |f: impl IntoBoxed<dyn Fn() -> bool>| f.into_boxed()))]
    when: Box<dyn Fn() -> bool>,
    children: Box<dyn Fn() -> Box<dyn View<B>>>,
    __caller_id: u32,
    #[builder(default)]
    _phantom: PhantomData<B>,
}

impl<B> Case<B>
where
    B: Backend + 'static,
{
    #[allow(clippy::new_ret_no_self, clippy::type_complexity)]
    pub fn new(
        children: impl IntoBoxed<dyn Fn() -> Box<dyn View<B>>>,
    ) -> CaseBuilder<
        B,
        (
            (),
            (Box<(dyn Fn() -> Box<(dyn tui_rsx::View<B> + 'static)> + 'static)>,),
            (),
            (),
        ),
    > {
        Self::builder().children(children.into_boxed())
    }
}

#[component]
pub fn Switch<B>(_cx: Scope, #[prop(children)] children: Vec<Case<B>>) -> impl View<B>
where
    B: Backend + 'static,
{
    move || {
        for child in &children {
            if (child.when)() {
                return (child.children)();
            }
        }
        panic!("No cases returned true")
    }
}
