// use rooibos_reactive::{
//     create_rw_signal, create_signal, provide_context, store_value, use_context, IntoSignal,
//     ReadSignal, RwSignal, Signal, SignalGet, SignalGetUntracked, SignalSetUntracked,
// SignalUpdate,     SignalUpdateUntracked, StoredValue, WriteSignal,
// };
// use typed_builder::TypedBuilder;
// use url::Url;

// use crate::prelude::*;

// pub trait IntoDomNodeFn {
//     fn into_dom_node_fn(self) -> Box<dyn FnOnce() -> DomNode>;
// }

// impl<F> IntoDomNodeFn for F
// where
//     F: Fn() -> DomNode + 'static,
// {
//     fn into_dom_node_fn(self) -> Box<dyn FnOnce() -> DomNode> {
//         Box::new(self)
//     }
// }

// #[derive(TypedBuilder, ComponentChildren)]
// pub struct Route {
//     #[builder(setter(transform = |p: impl Into<String>| p.into()))]
//     path: String,
//     #[children]
//     #[builder(setter(transform = |f: impl IntoDomNodeFn| f.into_dom_node_fn()))]
//     children: Box<dyn FnOnce() -> DomNode>,
// }

// #[derive(Clone)]
// pub struct RouteContext {
//     history: RwSignal<Vec<Url>>,
//     current_route: Signal<Url>,
//     router: StoredValue<matchit::Router<usize>>,
// }

// impl RouteContext {
//     pub fn push(&self, route: impl AsRef<str>) {
//         let url = Url::options()
//             .base_url(Some(&"app://".parse().unwrap()))
//             .parse(route.as_ref())
//             .unwrap();
//         self.history.update(|h| {
//             h.push(url);
//         });
//     }

//     pub fn pop(&self) {
//         self.history.update(|h| {
//             h.pop();
//         });
//     }

//     pub fn current_route(&self) -> Url {
//         self.current_route.get()
//     }

//     pub fn use_param(&self, param: impl Into<String>) -> Signal<Option<String>> {
//         let router = self.router.get_value();
//         let param = param.into();
//         let current_route = self.current_route;
//         (move || {
//             let route = current_route.get();

//             let params = router.at(route.path()).unwrap().params;
//             params.get(&param).map(|s| s.to_owned())
//         })
//         .into_signal()
//     }

//     pub fn use_query(&self, query: impl Into<String>) -> Signal<Option<String>> {
//         let route = self.current_route.get();
//         let query = query.into();
//         (move || {
//             route.query_pairs().find_map(|q| {
//                 if q.0 == query {
//                     Some(q.1.to_string())
//                 } else {
//                     None
//                 }
//             })
//         })
//         .into_signal()
//     }
// }

// pub fn use_router() -> RouteContext {
//     use_context::<RouteContext>().unwrap()
// }

// fn init_router(initial: String) {
//     let history = create_rw_signal(vec![]);
//     let context = RouteContext {
//         history,
//         router: store_value(matchit::Router::new()),
//         current_route: (move || {
//             let h = history.get();
//             h.last().cloned().unwrap()
//         })
//         .into_signal(),
//     };
//     context.push(initial);
//     provide_context(context);
// }

// #[component]
// pub fn Router(
//     #[prop(children)] routes: Vec<Route>,
//     #[prop(into, default = "/".to_owned())] initial: String,
// ) -> impl IntoView {
//     init_router(initial);
//     let router_ctx = use_router();

//     router_ctx.router.update_value(|mut r| {
//         for (i, child) in routes.iter().enumerate() {
//             r.insert(&child.path, i).unwrap();
//         }
//     });
//     // let children_cache = store_value::<Vec<Option<DomNode>>>(vec![None; routes.len()]);
//     // let children: Vec<_> = routes.into_iter().map(|c| Some(c.children)).collect();
//     let prev = rooibos_reactive::SpecialNonReactiveZone::enter();
//     let children: Vec<_> = routes.into_iter().map(|r| (r.children)()).collect();
//     rooibos_reactive::SpecialNonReactiveZone::exit(prev);

//     DynChild::new(move || {
//         let r = router_ctx.router.get_value();
//         let cur = router_ctx.current_route();
//         let index = r.at(cur.path()).unwrap().value;
//         children[*index].clone()
//     })
// }
