use leptos_reactive::Scope;
use ratatui::backend::Backend;
use tui_rsx::prelude::*;
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

#[caller_id]
#[derive(TypedBuilder, ComponentChildren)]
pub struct Case<B>
where
    B: Backend + 'static,
{
    #[builder(setter(transform = |f: impl IntoBoxed<dyn Fn() -> bool>| f.into_boxed()))]
    when: Box<dyn Fn() -> bool>,
    #[children]
    #[builder(setter(transform = |f: impl IntoBoxed<dyn Fn() -> Box<dyn View<B>>>| f.into_boxed()))]
    children: Box<dyn Fn() -> Box<dyn View<B>>>,
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
