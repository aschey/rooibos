use reactive_graph::effect::RenderEffect;
use reactive_graph::owner::{provide_context, use_context, Owner, StoredValue};
use reactive_graph::signal::{signal, WriteSignal};
use reactive_graph::traits::{Get, Update, With};
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::{
    derive_signal, AnyViewState, ChildrenFnMut, DomNode, IntoChildrenFnMut, RooibosDom,
};
use tachys::renderer::Renderer;
use tachys::view::{Mountable, Render};
use url::Url;

pub struct Route {
    path: String,
    children: ChildrenFnMut,
}

impl Route {
    pub fn new(path: impl Into<String>, children: impl IntoChildrenFnMut) -> Self {
        Self {
            path: path.into(),
            children: children.into_children_fn_mut(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct RouteContext {
    set_history: WriteSignal<Vec<Url>>,
    current_route: Signal<Url>,
    router: StoredValue<matchit::Router<usize>>,
}

impl RouteContext {
    pub fn push(&self, route: impl AsRef<str>) {
        let url = Url::options()
            .base_url(Some(&"app://".parse().unwrap()))
            .parse(route.as_ref())
            .unwrap();
        self.set_history.update(|h| {
            // prevent storing duplicate history entries
            if let Some(last) = h.last() {
                if *last == url {
                    return;
                }
            }
            h.push(url);
        });
    }

    pub fn pop(&self) {
        self.set_history.update(|h| {
            // ensure we always have the initial history entry
            if h.len() > 1 {
                h.pop();
            }
        });
    }

    pub fn current_route(&self) -> Url {
        self.current_route.get()
    }

    pub fn try_use_param(&self, param: impl Into<String>) -> Signal<Option<String>> {
        let router = self.router.get_value();
        let param = param.into();
        let current_route = self.current_route;
        derive_signal!({
            let route = current_route.get();
            let params = router.at(route.path()).unwrap().params;
            params.get(&param).map(|s| s.to_owned())
        })
    }

    pub fn use_param(&self, param: impl Into<String>) -> Signal<String> {
        let param = self.try_use_param(param);
        derive_signal!(param.get().unwrap())
    }

    pub fn try_use_query(&self, query: impl Into<String>) -> Signal<Option<String>> {
        let query = query.into();
        let current_route = self.current_route;
        derive_signal!({
            current_route.with(|r| {
                r.query_pairs().find_map(|q| {
                    if q.0 == query {
                        Some(q.1.to_string())
                    } else {
                        None
                    }
                })
            })
        })
    }

    pub fn use_query(&self, query: impl Into<String>) -> Signal<String> {
        let query = self.try_use_query(query);
        derive_signal!(query.get().unwrap())
    }
}

pub fn use_router() -> RouteContext {
    use_context::<RouteContext>().expect("use_router called outside of router context")
}

fn init_router(initial: String) {
    let (history, set_history) = signal(vec![]);
    let context = RouteContext {
        set_history,
        router: StoredValue::new(matchit::Router::new()),
        current_route: derive_signal!(history.with(|h| h.last().cloned().unwrap())),
    };
    context.push(initial);
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
            initial: "/".to_string(),
        }
    }

    pub fn routes(mut self, routes: impl Into<Vec<Route>>) -> Self {
        self.routes = routes.into();
        self
    }

    pub fn initial(mut self, initial: impl Into<String>) -> Self {
        self.initial = initial.into();
        self
    }
}

pub struct RouterState {
    state: AnyViewState,
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

    fn insert_before_this(
        &self,
        parent: &<RooibosDom as tachys::prelude::Renderer>::Element,
        child: &mut dyn Mountable<RooibosDom>,
    ) -> bool {
        self.state.insert_before_this(parent, child)
    }
}

impl Render<RooibosDom> for Router {
    type State = RenderEffect<RouterState>;

    fn build(self) -> Self::State {
        init_router(self.initial);
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
