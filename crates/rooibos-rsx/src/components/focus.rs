use reactive::{IntoSignal, ReadSignal, SignalGet};
use rooibos_reactive::Scope;

use crate::cache::__WIDGET_CACHE;
use crate::prelude::*;

pub fn use_focus(cx: Scope) -> ReadSignal<bool> {
    let focused_scope = __WIDGET_CACHE.with(|c| c.focused_scope());
    (move || {
        focused_scope
            .get()
            .map(|focused| focused.id() == cx.id())
            .unwrap_or(false)
    })
    .derive_signal(cx)
}

pub struct FocusManager {}

impl FocusManager {
    pub fn focus_next(&self) {
        __WIDGET_CACHE.with(|c| c.focus_next())
    }

    pub fn focus_previous() {}
}

pub fn use_focus_manager() -> FocusManager {
    FocusManager {}
}

#[component]
pub fn FocusScope<V>(_cx: Scope, #[prop(children)] children: V) -> impl View
where
    V: LazyView + Clone,
{
    move || {
        let mut children = children.clone();
        view!(cx, <Column>{children}</Column>)
    }
}
