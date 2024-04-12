use std::sync::Arc;

use reactive_graph::effect::RenderEffect;
use reactive_graph::owner::{provide_context, use_context, StoredValue};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Update};
use reactive_graph::wrappers::read::Signal;
use rooibos_dom_macros::{component, ComponentChildren};
use tachys::renderer::Renderer;
use tachys::view::{Mountable, Render};
use typed_builder::TypedBuilder;
use url::Url;

use crate::{AnyViewState, ChildrenFnMut, DomNode, IntoChildrenMut, RenderAny, RooibosDom};

#[derive(TypedBuilder, ComponentChildren)]
pub struct Route {
    #[builder(setter(transform = |p: impl Into<String>| p.into()))]
    path: String,
    #[children]
    #[builder(setter(transform = |p: impl IntoChildrenMut| p.into_children_mut()))]
    children: ChildrenFnMut,
}

#[derive(Clone)]
pub struct RouteContext {
    history: RwSignal<Vec<Url>>,
    current_route: Signal<Url>,
    router: StoredValue<matchit::Router<usize>>,
}

impl RouteContext {
    pub fn push(&self, route: impl AsRef<str>) {
        let url = Url::options()
            .base_url(Some(&"app://".parse().unwrap()))
            .parse(route.as_ref())
            .unwrap();
        self.history.update(|h| {
            h.push(url);
        });
    }

    pub fn pop(&self) {
        self.history.update(|h| {
            h.pop();
        });
    }

    pub fn current_route(&self) -> Url {
        self.current_route.get()
    }

    pub fn use_param(&self, param: impl Into<String>) -> Signal<Option<String>> {
        let router = self.router.get().unwrap();
        let param = param.into();
        let current_route = self.current_route;
        Signal::derive(move || {
            let route = current_route.get();

            let params = router.at(route.path()).unwrap().params;
            params.get(&param).map(|s| s.to_owned())
        })
    }

    pub fn use_query(&self, query: impl Into<String>) -> Signal<Option<String>> {
        let route = self.current_route.get();
        let query = query.into();
        Signal::derive(move || {
            route.query_pairs().find_map(|q| {
                if q.0 == query {
                    Some(q.1.to_string())
                } else {
                    None
                }
            })
        })
    }
}

pub fn use_router() -> RouteContext {
    use_context::<RouteContext>().unwrap()
}

fn init_router(initial: String) {
    let history = RwSignal::new(vec![]);
    let context = RouteContext {
        history,
        router: StoredValue::new(matchit::Router::new()),
        current_route: Signal::derive(move || {
            let h = history.get();
            h.last().cloned().unwrap()
        }),
    };
    context.push(initial);
    provide_context(context);
}

pub struct Router {
    routes: Vec<Route>,
    initial: String,
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

    type FallibleState = RenderEffect<RouterState>;

    type AsyncOutput = ();

    fn build(self) -> Self::State {
        init_router(self.initial);
        let router_ctx = use_router();

        let mut routes = self.routes;

        router_ctx.router.update_value(|r| {
            for (i, child) in routes.iter().enumerate() {
                r.insert(&child.path, i).unwrap();
            }
        });

        RenderEffect::new(move |prev: Option<RouterState>| {
            let router = router_ctx.router.get().unwrap();
            let cur = router_ctx.current_route();
            let path = cur.path();
            let index = *router.at(path).unwrap().value;
            let view = (routes[index].children)();
            if let Some(mut router_state) = prev {
                if index == router_state.index {
                    view.rebuild(&mut router_state.state);
                    RouterState {
                        state: router_state.state,
                        index,
                        parent: router_state.parent,
                    }
                } else {
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
                RouterState {
                    state: view.build(),
                    index,
                    parent: None,
                }
            }
        })
    }

    fn rebuild(self, state: &mut Self::State) {}

    fn try_build(self) -> any_error::Result<Self::FallibleState> {
        todo!()
    }

    fn try_rebuild(self, state: &mut Self::FallibleState) -> any_error::Result<()> {
        todo!()
    }

    async fn resolve(self) -> Self::AsyncOutput {
        todo!()
    }
}

#[component]
pub fn Router(
    #[prop(children)] routes: Vec<Route>,
    #[prop(into, default = "/".to_owned())] initial: String,
) -> impl RenderAny {
    Router { routes, initial }
}
