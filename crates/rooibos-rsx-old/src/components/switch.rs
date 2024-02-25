use rooibos_reactive_old::Scope;
use typed_builder::TypedBuilder;

use crate::prelude::*;

#[caller_id]
#[derive(TypedBuilder, ComponentChildren)]
pub struct Case {
    #[builder(setter(transform = |f: impl IntoBoxed<dyn Fn() -> bool>| f.into_boxed()))]
    when: Box<dyn Fn() -> bool>,
    #[children]
    #[builder(setter(transform = |f: impl IntoBoxedViewFn| f.into_boxed_view_fn()))]
    children: Box<dyn Fn() -> Box<dyn View>>,
}

#[component]
pub fn Switch(
    _cx: Scope,
    #[prop(children)] children: Vec<Case>,
    #[prop(default = true)] lazy: bool,
) -> impl View {
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
