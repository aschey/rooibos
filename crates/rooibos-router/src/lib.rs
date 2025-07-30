use std::borrow::Cow;

use reactive_graph::computed::Memo;
use reactive_graph::owner::{Owner, StoredValue, provide_context, use_context};
use reactive_graph::signal::{WriteSignal, signal};
use reactive_graph::traits::{Get, GetValue, Update, UpdateValue, With};
use reactive_graph::wrappers::read::Signal;
use rooibos_reactive::derive_signal;
use rooibos_reactive::dom::{ChildrenFnMut, IntoChildrenFnMut};
pub use rooibos_router_macros::*;
use url::Url;

pub trait ToRoute {
    fn to_route(&self) -> String;
}

impl ToRoute for Box<dyn ToRoute> {
    fn to_route(&self) -> String {
        (**self).to_route()
    }
}

pub trait ToRouteTemplateStatic {
    fn to_route_template() -> &'static str;
}

pub trait ToRouteTemplate {
    fn to_route_template(&self) -> &'static str;
}

pub struct Param(pub Cow<'static, str>);

pub struct Query(pub Cow<'static, str>);

pub struct Route {
    path: String,
    children: ChildrenFnMut,
}

pub trait RouteFromStatic {
    fn new<T: ToRouteTemplateStatic>(children: impl IntoChildrenFnMut) -> Self;
}

pub trait RouteFrom {
    fn new(route: impl ToRouteTemplate, children: impl IntoChildrenFnMut) -> Self;
}

impl RouteFromStatic for Route {
    fn new<T: ToRouteTemplateStatic>(children: impl IntoChildrenFnMut) -> Self {
        Self {
            path: T::to_route_template().to_string(),
            children: children.into_children_fn_mut(),
        }
    }
}

impl RouteFrom for Route {
    fn new(route: impl ToRouteTemplate, children: impl IntoChildrenFnMut) -> Self {
        Self {
            path: route.to_route_template().to_string(),
            children: children.into_children_fn_mut(),
        }
    }
}

pub struct DefaultRoute;

impl ToRoute for DefaultRoute {
    fn to_route(&self) -> String {
        "/".to_string()
    }
}

impl ToRouteTemplateStatic for DefaultRoute {
    fn to_route_template() -> &'static str {
        "/"
    }
}

#[derive(Clone, Copy)]
pub struct RouteContext {
    set_history: WriteSignal<Vec<Url>>,
    current_route: Memo<Url>,
    router: StoredValue<matchit::Router<usize>>,
    set_buffer: WriteSignal<Vec<Url>>,
    can_go_forward: Signal<bool>,
    can_go_back: Signal<bool>,
}

impl RouteContext {
    pub fn push(&self, route: impl ToRoute) {
        self.push_str(&route.to_route())
    }

    fn push_str(&self, route: &str) {
        let url = Url::options()
            .base_url(Some(&"app://".parse().unwrap()))
            .parse(route)
            .unwrap();
        self.push_url(url, true);
    }

    fn push_url(&self, url: Url, clear_buffer: bool) {
        self.set_history.update(|h| {
            // prevent storing duplicate history entries
            if let Some(last) = h.last()
                && *last == url
            {
                return;
            }
            h.push(url);
            if clear_buffer {
                self.set_buffer.update(|b| b.clear());
            }
        });
    }

    fn replace_root(&self, new_root: &str) {
        self.set_history.update(|h| {
            h.clear();
        });
        self.push_str(new_root);
    }

    pub fn back(&self) {
        self.set_history.update(|h| {
            // ensure we always have the initial history entry
            if h.len() > 1 {
                self.set_buffer.update(|b| b.push(h.pop().unwrap()));
            }
        });
    }

    pub fn forward(&self) {
        self.set_buffer.update(|b| {
            if let Some(entry) = b.pop() {
                self.push_url(entry, false);
            }
        })
    }

    pub fn current_route(&self) -> Url {
        self.current_route.get()
    }

    pub fn try_use_param(&self, param: Param) -> Signal<Option<String>> {
        let router = self.router.get_value();
        let current_route = self.current_route;
        derive_signal!({
            let route = current_route.get();
            let params = router.at(route.path()).unwrap().params;
            params.get(&param.0).map(|s| s.to_owned())
        })
    }

    pub fn use_param(&self, param: Param) -> Signal<String> {
        let param = self.try_use_param(param);
        derive_signal!(param.get().unwrap())
    }

    pub fn try_use_query(&self, query: Query) -> Signal<Option<String>> {
        let current_route = self.current_route;
        derive_signal!({
            current_route.with(|r| {
                r.query_pairs().find_map(|q| {
                    if q.0 == query.0 {
                        Some(q.1.to_string())
                    } else {
                        None
                    }
                })
            })
        })
    }

    pub fn use_query(&self, query: Query) -> Signal<String> {
        let query = self.try_use_query(query);
        derive_signal!(query.get().unwrap())
    }

    pub fn can_go_forward(&self) -> Signal<bool> {
        self.can_go_forward
    }

    pub fn can_go_back(&self) -> Signal<bool> {
        self.can_go_back
    }
}

pub fn use_route_context() -> Option<RouteContext> {
    use_context::<RouteContext>()
}

fn use_route_context_internal(initial: &str) -> RouteContext {
    if let Some(context) = use_context::<RouteContext>() {
        context
    } else {
        init_router(initial)
    }
}

fn init_router(initial: &str) -> RouteContext {
    let (history, set_history) = signal(vec![]);
    let (buffer, set_buffer) = signal(vec![]);
    let context = RouteContext {
        set_history,
        router: StoredValue::new(matchit::Router::new()),
        current_route: Memo::new(move |_| history.with(|h| h.last().cloned().unwrap())),
        set_buffer,
        can_go_forward: derive_signal!(buffer.with(|b| !b.is_empty())),
        can_go_back: derive_signal!(history.with(|h| { h.len() > 1 })),
    };
    context.push_str(initial);
    provide_context(context);
    context
}

pub fn use_router() -> (Router, RouteContext) {
    (Router::new(), use_route_context_internal("/"))
}

pub struct Router {
    routes: Vec<Route>,
    initial: String,
}

impl Router {
    fn new() -> Self {
        Self {
            routes: Vec::default(),
            initial: DefaultRoute.to_route(),
        }
    }

    pub fn routes(mut self, routes: impl Into<Vec<Route>>) -> Self {
        self.routes = routes.into();
        self
    }

    pub fn initial(mut self, initial: impl ToRoute) -> Self {
        self.initial = initial.to_route();
        self
    }

    pub fn render(self) -> impl rooibos_reactive::dom::RenderAny {
        let route_ctx = use_route_context_internal(&self.initial);
        route_ctx.replace_root(&self.initial);

        let mut routes = self.routes;

        route_ctx.router.update_value(|r| {
            for (i, child) in routes.iter().enumerate() {
                r.insert(&child.path, i).unwrap();
            }
        });
        let owner = Owner::current().unwrap();
        let route_index = Memo::new(move |_| {
            let current_route = route_ctx.current_route();
            let path = current_route.path();
            let router = route_ctx.router.get_value();
            *router.at(path).unwrap().value
        });
        move || {
            let index = route_index.get();
            owner.with(|| (routes[index].children)())
        }
    }
}
