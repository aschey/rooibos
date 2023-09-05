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
    B: Backend + 'static,
    V: LazyView<B> + Clone + 'static,
{
    let inverse_y = (100 - percent_y) / 2;
    let inverse_x = (100 - percent_x) / 2;
    move || {
        let mut children = children.clone();
        view! { cx,
            <column>
                <row percentage=inverse_y />
                <row percentage=percent_y>
                    <column percentage=inverse_x />
                    <column percentage=percent_x>
                        <overlay>
                            <clear/>
                            {children}
                        </overlay>
                    </column>
                    <column percentage=inverse_x />
                </row>
                <row percentage=inverse_y />
            </column>
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
    B: Backend + 'static,
    W: Fn() -> bool + 'static,
    F1: Fn() -> V1 + 'static,
    F2: Fn() -> V2 + 'static,
    V1: View<B> + 'static,
    V2: View<B> + 'static,
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
    B: Backend + 'static,
{
    #[builder(setter(transform = |f: impl IntoBoxed<dyn Fn() -> bool>| f.into_boxed()))]
    when: Box<dyn Fn() -> bool>,
    #[children]
    #[builder(setter(transform = |f: impl IntoBoxed<dyn Fn() -> Box<dyn View<B>>>| f.into_boxed()))]
    children: Box<dyn Fn() -> Box<dyn View<B>>>,
}

#[component]
pub fn Switch<B>(
    _cx: Scope,
    #[prop(children)] children: Vec<Case<B>>,
    #[prop(default = false)] eager: bool,
) -> impl View<B>
where
    B: Backend + 'static,
{
    move || {
        let mut res = None;
        for child in &children {
            match ((child.when)(), eager, &res) {
                (true, true, None) => {
                    res = Some((child.children)());
                }
                (false, true, _) => {
                    (child.children)();
                }
                (true, false, _) => return (child.children)(),
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
