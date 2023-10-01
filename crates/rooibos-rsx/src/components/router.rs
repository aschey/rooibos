use reactive::{
    create_signal, provide_global_context, store_value, use_context, IntoSignal, ReadSignal,
    Signal, SignalGet, SignalUpdate, StoredValue,
};
use rooibos_reactive::Scope;
use typed_builder::TypedBuilder;

use crate::prelude::*;

#[caller_id]
#[derive(TypedBuilder, ComponentChildren)]
pub struct Route {
    #[builder(setter(transform = |p: impl Into<String>| p.into()))]
    path: String,
    #[children]
    #[builder(setter(transform = |f: impl IntoBoxedViewFn| f.into_boxed_view_fn()))]
    children: Box<dyn Fn() -> Box<dyn View>>,
}

#[derive(Clone)]
pub struct RouteContext {
    history: Signal<Vec<String>>,
    current_route: ReadSignal<String>,
    router: StoredValue<matchit::Router<usize>>,
}

impl RouteContext {
    pub fn push(&self, route: impl Into<String>) {
        self.history.update(|mut h| {
            h.push(route.into());
            h
        });
    }

    pub fn pop(&self) {
        self.history.update(|mut h| {
            h.pop();
            h
        });
    }

    pub fn current_route(&self) -> String {
        self.current_route.get()
    }

    pub fn get_param(&self, param: impl AsRef<str>) -> Option<String> {
        let router = self.router.get_value();
        let route = self.current_route.get();
        let params = router.at(&route).unwrap().params;
        params.get(param).map(|s| s.to_owned())
    }
}

pub fn use_router(cx: Scope) -> RouteContext {
    use_context::<RouteContext>(cx)
}

pub fn init_router(cx: Scope) {
    let history = create_signal(cx, vec![]);
    provide_global_context(
        cx,
        RouteContext {
            history,
            router: store_value(cx, matchit::Router::new()),
            current_route: (move || {
                let h = history.get();
                h.last().cloned().unwrap()
            })
            .derive_signal(cx),
        },
    );
}

#[component]
pub fn Router(
    cx: Scope,
    #[prop(children)] children: Vec<Route>,
    #[prop(into, default = "/".to_string())] initial: String,
    #[prop(default = true)] lazy: bool,
) -> impl View {
    let router_ctx = use_router(cx);
    router_ctx.push(initial);
    router_ctx.router.update_value(|mut r| {
        for (i, child) in children.iter().enumerate() {
            r.insert(&child.path, i).unwrap();
        }
        r.clone()
    });

    move || {
        if !lazy {
            for child in &children {
                (child.children)();
            }
        }

        let r = router_ctx.router.get_value();
        let cur = router_ctx.current_route();
        let index = r.at(&cur).unwrap().value;
        (children[*index].children)()
    }
}
