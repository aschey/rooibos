// use std::any::{Any, TypeId};
// use std::fmt::Debug;
// use std::marker::PhantomData;

// use tachys::view::Mountable;

// use crate::{DomNode, RooibosDom};

// pub struct AnyView {
//     type_id: TypeId,
//     value: Box<dyn Any + Send>,
//     build: fn(Box<dyn Any>) -> AnyViewState,
//     rebuild: fn(TypeId, Box<dyn Any>, &mut AnyViewState),
// }

// pub struct AnyViewState {
//     type_id: TypeId,
//     state: Box<dyn Any>,
//     unmount: fn(&mut dyn Any),
//     mount: fn(&mut dyn Any, parent: &DomNode, marker: Option<&DomNode>),
//     insert_before_this:
//         fn(&dyn Any, parent: &DomNode, child: &mut dyn Mountable<RooibosDom>) -> bool,
// }

// impl Debug for AnyViewState {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("AnyViewState")
//             .field("type_id", &self.type_id)
//             .field("state", &self.state)
//             .field("unmount", &self.unmount)
//             .field("mount", &self.mount)
//             .field("insert_before_this", &self.insert_before_this)
//             .finish()
//     }
// }

// pub trait IntoAny {
//     fn into_any(self) -> AnyView;
// }

// fn mount_any(state: &mut dyn Any, parent: &DomNode, marker: Option<&DomNode>) {
//     let state = state
//         .downcast_mut::<DomNode>()
//         .expect("AnyViewState::as_mountable couldn't downcast state");
//     state.mount(parent, marker)
// }

// fn unmount_any(state: &mut dyn Any) {
//     let state = state
//         .downcast_mut::<DomNode>()
//         .expect("AnyViewState::unmount couldn't downcast state");
//     state.unmount();
// }

// fn insert_before_this<R, T>(
//     state: &dyn Any,
//     parent: &R::Element,
//     child: &mut dyn Mountable<R>,
// ) -> bool
// where
//     T: Render<R>,
//     T::State: 'static,
//     R: Renderer + 'static,
// {
//     let state = state
//         .downcast_ref::<T::State>()
//         .expect("AnyViewState::opening_node couldn't downcast state");
//     state.insert_before_this(parent, child)
// }

// impl<T, R> IntoAny<R> for T
// where
//     T: Send,
//     T: RenderHtml<R> + 'static,
//     T::State: 'static,
//     R: Renderer + 'static,
// {
//     // inlining allows the compiler to remove the unused functions
//     // i.e., doesn't ship HTML-generating code that isn't used
//     #[inline(always)]
//     fn into_any(self) -> AnyView<R> {
//         #[cfg(feature = "ssr")]
//         let html_len = self.html_len();

//         let value = Box::new(self) as Box<dyn Any + Send>;

//         #[cfg(feature = "ssr")]
//         let to_html = |value: Box<dyn Any>, buf: &mut String, position: &mut Position| {
//             let value = value
//                 .downcast::<T>()
//                 .expect("AnyView::to_html could not be downcast");
//             value.to_html_with_buf(buf, position);
//         };
//         #[cfg(feature = "ssr")]
//         let to_html_async =
//             |value: Box<dyn Any>, buf: &mut StreamBuilder, position: &mut Position| {
//                 let value = value
//                     .downcast::<T>()
//                     .expect("AnyView::to_html could not be downcast");
//                 value.to_html_async_with_buf::<false>(buf, position);
//             };
//         #[cfg(feature = "ssr")]
//         let to_html_async_ooo =
//             |value: Box<dyn Any>, buf: &mut StreamBuilder, position: &mut Position| {
//                 let value = value
//                     .downcast::<T>()
//                     .expect("AnyView::to_html could not be downcast");
//                 value.to_html_async_with_buf::<true>(buf, position);
//             };
//         let build = |value: Box<dyn Any>| {
//             let value = value
//                 .downcast::<T>()
//                 .expect("AnyView::build couldn't downcast");
//             let state = Box::new(value.build());

//             AnyViewState {
//                 type_id: TypeId::of::<T>(),
//                 state,
//                 rndr: PhantomData,
//                 mount: mount_any::<R, T>,
//                 unmount: unmount_any::<R, T>,
//                 insert_before_this: insert_before_this::<R, T>,
//             }
//         };
//         #[cfg(feature = "hydrate")]
//         let hydrate_from_server =
//             |value: Box<dyn Any>, cursor: &Cursor<R>, position: &PositionState| {
//                 let value = value
//                     .downcast::<T>()
//                     .expect("AnyView::hydrate_from_server couldn't downcast");
//                 let state = Box::new(value.hydrate::<true>(cursor, position));

//                 AnyViewState {
//                     type_id: TypeId::of::<T>(),
//                     state,
//                     rndr: PhantomData,
//                     mount: mount_any::<R, T>,
//                     unmount: unmount_any::<R, T>,
//                     insert_before_this: insert_before_this::<R, T>,
//                 }
//             };
//         #[cfg(feature = "hydrate")]
//         let hydrate_from_template =
//             |value: Box<dyn Any>, cursor: &Cursor<R>, position: &PositionState| {
//                 let value = value
//                     .downcast::<T>()
//                     .expect("AnyView::hydrate_from_server couldn't downcast");
//                 let state = Box::new(value.hydrate::<true>(cursor, position));

//                 AnyViewState {
//                     type_id: TypeId::of::<T>(),
//                     state,
//                     rndr: PhantomData,
//                     mount: mount_any::<R, T>,
//                     unmount: unmount_any::<R, T>,
//                     insert_before_this: insert_before_this::<R, T>,
//                 }
//             };
//         let rebuild = |new_type_id: TypeId, value: Box<dyn Any>, state: &mut AnyViewState<R>| {
//             let value = value
//                 .downcast::<T>()
//                 .expect("AnyView::rebuild couldn't downcast value");
//             if new_type_id == state.type_id {
//                 let state = state
//                     .state
//                     .downcast_mut()
//                     .expect("AnyView::rebuild couldn't downcast state");
//                 value.rebuild(state);
//             } else {
//                 let new = value.into_any().build();

//                 // TODO mount new state
//                 /* R::mount_before(&mut new, state.placeholder.as_ref()); */
//                 state.unmount();
//                 *state = new;
//             }
//         };
//         AnyView {
//             type_id: TypeId::of::<T>(),
//             value,
//             build,
//             rebuild,
//             #[cfg(feature = "ssr")]
//             html_len,
//             #[cfg(feature = "ssr")]
//             to_html,
//             #[cfg(feature = "ssr")]
//             to_html_async,
//             #[cfg(feature = "ssr")]
//             to_html_async_ooo,
//             #[cfg(feature = "hydrate")]
//             hydrate_from_server,
//             #[cfg(feature = "hydrate")]
//             hydrate_from_template,
//         }
//     }
// }

// impl<R> Render<R> for AnyView<R>
// where
//     R: Renderer + 'static,
// {
//     type State = AnyViewState<R>;
//     type FallibleState = Self::State;
//     type AsyncOutput = Self;

//     fn build(self) -> Self::State {
//         (self.build)(self.value)
//     }

//     fn rebuild(self, state: &mut Self::State) {
//         (self.rebuild)(self.type_id, self.value, state)
//     }

//     fn try_build(self) -> any_error::Result<Self::FallibleState> {
//         todo!()
//     }

//     fn try_rebuild(self, _state: &mut Self::FallibleState) -> any_error::Result<()> {
//         todo!()
//     }

//     async fn resolve(self) -> Self::AsyncOutput {
//         // we probably do need a function for this
//         todo!()
//     }
// }

// impl<R> RenderHtml<R> for AnyView<R>
// where
//     R: Renderer + 'static,
// {
//     const MIN_LENGTH: usize = 0;

//     fn to_html_with_buf(self, buf: &mut String, position: &mut Position) {
//         #[cfg(feature = "ssr")]
//         (self.to_html)(self.value, buf, position);
//         #[cfg(not(feature = "ssr"))]
//         {
//             _ = buf;
//             _ = position;
//             panic!("You are rendering AnyView to HTML without the `ssr` feature enabled.");
//         }
//     }

//     fn to_html_async_with_buf<const OUT_OF_ORDER: bool>(
//         self,
//         buf: &mut StreamBuilder,
//         position: &mut Position,
//     ) where
//         Self: Sized,
//     {
//         #[cfg(feature = "ssr")]
//         if OUT_OF_ORDER {
//             (self.to_html_async_ooo)(self.value, buf, position);
//         } else {
//             (self.to_html_async)(self.value, buf, position);
//         }
//         #[cfg(not(feature = "ssr"))]
//         {
//             _ = buf;
//             _ = position;
//             panic!("You are rendering AnyView to HTML without the `ssr` feature enabled.");
//         }
//     }

//     fn hydrate<const FROM_SERVER: bool>(
//         self,
//         cursor: &Cursor<R>,
//         position: &PositionState,
//     ) -> Self::State {
//         #[cfg(feature = "hydrate")]
//         if FROM_SERVER {
//             (self.hydrate_from_server)(self.value, cursor, position)
//         } else {
//             (self.hydrate_from_template)(self.value, cursor, position)
//         }
//         #[cfg(not(feature = "hydrate"))]
//         {
//             _ = cursor;
//             _ = position;
//             panic!("You are trying to hydrate AnyView without the `hydrate` feature enabled.");
//         }
//     }
// }

// impl<R> Mountable<R> for AnyViewState<R>
// where
//     R: Renderer + 'static,
// {
//     fn unmount(&mut self) {
//         (self.unmount)(&mut *self.state)
//     }

//     fn mount(&mut self, parent: &R::Element, marker: Option<&R::Node>) {
//         (self.mount)(&mut *self.state, parent, marker)
//     }

//     fn insert_before_this(
//         &self,
//         parent: &<R as Renderer>::Element,
//         child: &mut dyn Mountable<R>,
//     ) -> bool {
//         (self.insert_before_this)(self, parent, child)
//     }
// }
