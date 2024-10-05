use std::borrow::Cow;

use reactive_graph::effect::RenderEffect;
use reactive_graph::owner::{Owner, StoredValue, provide_context, use_context};
use reactive_graph::signal::{WriteSignal, signal};
use reactive_graph::traits::{Get, Update, With};
use reactive_graph::wrappers::read::Signal;
use rooibos_reactive::{ChildrenFnMut, DomNode, IntoChildrenFnMut, RooibosDom, derive_signal};
pub use rooibos_router_macros::*;
use tachys::renderer::Renderer;
use tachys::view::any_view::AnyViewState;
use tachys::view::{Mountable, Render};
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
    current_route: Signal<Url>,
    router: StoredValue<matchit::Router<usize>>,
    set_buffer: WriteSignal<Vec<Url>>,
    can_go_forward: Signal<bool>,
    can_go_back: Signal<bool>,
}

impl RouteContext {
    pub fn push(&self, route: impl ToRoute) {
        self.push_str(route.to_route())
    }

    fn push_str(&self, route: String) {
        let url = Url::options()
            .base_url(Some(&"app://".parse().unwrap()))
            .parse(&route)
            .unwrap();
        self.push_url(url, true);
    }

    fn push_url(&self, url: Url, clear_buffer: bool) {
        self.set_history.update(|h| {
            // prevent storing duplicate history entries
            if let Some(last) = h.last() {
                if *last == url {
                    return;
                }
            }
            h.push(url);
            if clear_buffer {
                self.set_buffer.update(|b| b.clear());
            }
        });
    }

    fn replace_root(&self, new_root: String) {
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

pub fn use_router() -> RouteContext {
    use_context::<RouteContext>().expect("use_router called outside of router context")
}

pub fn provide_router() {
    init_router("/".to_string());
}

fn init_router(initial: String) {
    let (history, set_history) = signal(vec![]);
    let (buffer, set_buffer) = signal(vec![]);
    let context = RouteContext {
        set_history,
        router: StoredValue::new(matchit::Router::new()),
        current_route: derive_signal!(history.with(|h| h.last().cloned().unwrap())),
        set_buffer,
        can_go_forward: derive_signal!(buffer.with(|b| !b.is_empty())),
        can_go_back: derive_signal!(history.with(|h| h.len() > 1)),
    };
    context.push_str(initial);
    provide_context(context);
}

pub struct Router {
    routes: Vec<Route>,
    initial: String,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
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
}

pub struct RouterState {
    state: AnyViewState<RooibosDom>,
    parent: Option<DomNode>,
    index: usize,
}

impl Mountable<RooibosDom> for RouterState {
    fn unmount(&mut self) {
        self.state.unmount();
    }

    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        self.parent = Some(parent.clone());
        self.state.mount(parent, marker)
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        self.state.insert_before_this(child)
    }
}

impl Render<RooibosDom> for Router {
    type State = RenderEffect<RouterState>;

    fn build(self) -> Self::State {
        if let Some(context) = use_context::<RouteContext>() {
            context.replace_root(self.initial);
        } else {
            init_router(self.initial);
        }
        let router_ctx = use_router();

        let mut routes = self.routes;

        router_ctx.router.update_value(|r| {
            for (i, child) in routes.iter().enumerate() {
                r.insert(&child.path, i).unwrap();
            }
        });

        let parent = Owner::current().expect("no reactive owner");

        RenderEffect::new(move |prev: Option<RouterState>| {
            let router = router_ctx.router.get_value();
            let cur = router_ctx.current_route();
            let path = cur.path();
            let index = *router.at(path).unwrap().value;

            if let Some(mut router_state) = prev {
                if index == router_state.index {
                    RouterState {
                        state: router_state.state,
                        index,
                        parent: router_state.parent,
                    }
                } else {
                    let view = parent.with(|| (routes[index].children)());
                    router_state.state.unmount();
                    let mut new = view.build();
                    if let Some(parent) = &router_state.parent {
                        new.mount(parent, None);
                    }

                    RouterState {
                        state: new,
                        index,
                        parent: router_state.parent,
                    }
                }
            } else {
                let view = parent.with(|| (routes[index].children)());
                RouterState {
                    state: view.build(),
                    index,
                    parent: None,
                }
            }
        })
    }

    fn rebuild(self, _state: &mut Self::State) {}
}
