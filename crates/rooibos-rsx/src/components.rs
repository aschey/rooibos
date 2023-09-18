use rooibos_reactive::Scope;
use typed_builder::TypedBuilder;

use crate::prelude::*;

#[component]
pub fn Popup<B, V>(
    cx: Scope,
    #[prop(children)] children: V,
    percent_x: u16,
    percent_y: u16,
) -> impl View<B>
where
    B: Backend,
    V: LazyView<B> + Clone,
{
    let inverse_y = (100 - percent_y) / 2;
    let inverse_x = (100 - percent_x) / 2;
    move || {
        let mut children = children.clone();
        view! { cx,
            <Column>
                <Row v:percentage=inverse_y />
                <Row v:percentage=percent_y>
                    <Column v:percentage=inverse_x />
                    <Column v:percentage=percent_x>
                        <Overlay>
                            <Clear/>
                            {children}
                        </Overlay>
                    </Column>
                    <Column v:percentage=inverse_x />
                </Row>
                <Row v:percentage=inverse_y />
            </Column>
        }
    }
}

#[component]
pub fn Show<B, F1, F2, V1, V2, W>(
    _cx: Scope,
    #[prop(children)] children: F1,
    when: W,
    fallback: F2,
    #[prop(default = false)] eager: bool,
) -> impl View<B>
where
    B: Backend,
    W: Fn() -> bool + 'static,
    F1: Fn() -> V1 + 'static,
    F2: Fn() -> V2 + 'static,
    V1: View<B>,
    V2: View<B>,
{
    move || match (when(), eager) {
        (true, true) => {
            fallback();
            children().into_boxed_view()
        }
        (true, false) => children().into_boxed_view(),
        (false, true) => {
            children().into_boxed_view();
            fallback().into_boxed_view()
        }
        (false, false) => fallback().into_boxed_view(),
    }
}

#[caller_id]
#[derive(TypedBuilder, ComponentChildren)]
pub struct Case<B>
where
    B: Backend,
{
    #[builder(setter(transform = |f: impl IntoBoxed<dyn Fn() -> bool>| f.into_boxed()))]
    when: Box<dyn Fn() -> bool>,
    #[children]
    #[builder(setter(transform = |f: impl IntoBoxedLazyView<B>| f.into_boxed_lazy_view()))]
    children: Box<dyn Fn() -> Box<dyn View<B>>>,
}

#[component]
pub fn Switch<B>(
    _cx: Scope,
    #[prop(children)] children: Vec<Case<B>>,
    #[prop(default = true)] lazy: bool,
) -> impl View<B>
where
    B: Backend,
{
    move || {
        let mut res = None;
        for child in &children {
            match ((child.when)(), lazy, &res) {
                (true, false, None) => {
                    res = Some((child.children)());
                }
                (false, false, _) => {
                    (child.children)();
                }
                (true, true, _) => return (child.children)(),
                _ => {}
            }
        }
        if let Some(res) = res {
            res
        } else {
            panic!("No cases returned true")
        }
    }
}
