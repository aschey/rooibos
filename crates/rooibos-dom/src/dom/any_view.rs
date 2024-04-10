use std::any::{Any, TypeId};
use std::fmt::Debug;

use tachys::view::{Mountable, Render};

use crate::{DomNode, RooibosDom};

pub struct AnyView {
    type_id: TypeId,
    value: Box<dyn Any>,
    build: fn(Box<dyn Any>) -> AnyViewState,
    rebuild: fn(TypeId, Box<dyn Any>, &mut AnyViewState),
}

pub struct AnyViewState {
    type_id: TypeId,
    state: Box<dyn Any>,
    unmount: fn(&mut dyn Any),
    mount: fn(&mut dyn Any, parent: &DomNode, marker: Option<&DomNode>),
    insert_before_this:
        fn(&dyn Any, parent: &DomNode, child: &mut dyn Mountable<RooibosDom>) -> bool,
}

impl Debug for AnyViewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyViewState")
            .field("type_id", &self.type_id)
            .field("state", &self.state)
            .field("unmount", &self.unmount)
            .field("mount", &self.mount)
            .field("insert_before_this", &self.insert_before_this)
            .finish()
    }
}

pub trait IntoAny {
    fn into_any(self) -> AnyView;
}

fn mount_any<T>(state: &mut dyn Any, parent: &DomNode, marker: Option<&DomNode>)
where
    T: Render<RooibosDom>,
    T::State: 'static,
{
    let state = state
        .downcast_mut::<T::State>()
        .expect("AnyViewState::as_mountable couldn't downcast state");
    state.mount(parent, marker)
}

fn unmount_any<T>(state: &mut dyn Any)
where
    T: Render<RooibosDom>,
    T::State: 'static,
{
    let state = state
        .downcast_mut::<T::State>()
        .expect("AnyViewState::unmount couldn't downcast state");
    state.unmount();
}

fn insert_before_this<T>(
    state: &dyn Any,
    parent: &DomNode,
    child: &mut dyn Mountable<RooibosDom>,
) -> bool
where
    T: Render<RooibosDom>,
    T::State: 'static,
{
    let state = state
        .downcast_ref::<T::State>()
        .expect("AnyViewState::opening_node couldn't downcast state");
    state.insert_before_this(parent, child)
}

impl<T> IntoAny for T
where
    T: Render<RooibosDom> + 'static,
    T::State: 'static,
{
    // inlining allows the compiler to remove the unused functions
    // i.e., doesn't ship HTML-generating code that isn't used
    #[inline(always)]
    fn into_any(self) -> AnyView {
        let value = Box::new(self) as Box<dyn Any>;

        let build = |value: Box<dyn Any>| {
            let value = value
                .downcast::<T>()
                .expect("AnyView::build couldn't downcast");
            let state = Box::new(value.build());

            AnyViewState {
                type_id: TypeId::of::<T>(),
                state,

                mount: mount_any::<T>,
                unmount: unmount_any::<T>,
                insert_before_this: insert_before_this::<T>,
            }
        };

        let rebuild = |new_type_id: TypeId, value: Box<dyn Any>, state: &mut AnyViewState| {
            let value = value
                .downcast::<T>()
                .expect("AnyView::rebuild couldn't downcast value");
            if new_type_id == state.type_id {
                let state = state
                    .state
                    .downcast_mut()
                    .expect("AnyView::rebuild couldn't downcast state");
                value.rebuild(state);
            } else {
                let new = value.into_any().build();

                // TODO mount new state
                /* R::mount_before(&mut new, state.placeholder.as_ref()); */
                state.unmount();
                *state = new;
            }
        };
        AnyView {
            type_id: TypeId::of::<T>(),
            value,
            build,
            rebuild,
        }
    }
}

impl Render<RooibosDom> for AnyView {
    type State = AnyViewState;
    type FallibleState = Self::State;
    type AsyncOutput = Self;

    fn build(self) -> Self::State {
        (self.build)(self.value)
    }

    fn rebuild(self, state: &mut Self::State) {
        (self.rebuild)(self.type_id, self.value, state)
    }

    fn try_build(self) -> any_error::Result<Self::FallibleState> {
        todo!()
    }

    fn try_rebuild(self, _state: &mut Self::FallibleState) -> any_error::Result<()> {
        todo!()
    }

    async fn resolve(self) -> Self::AsyncOutput {
        // we probably do need a function for this
        todo!()
    }
}

impl Mountable<RooibosDom> for AnyViewState {
    fn unmount(&mut self) {
        (self.unmount)(&mut *self.state)
    }

    fn mount(&mut self, parent: &DomNode, marker: Option<&DomNode>) {
        (self.mount)(&mut *self.state, parent, marker)
    }

    fn insert_before_this(&self, parent: &DomNode, child: &mut dyn Mountable<RooibosDom>) -> bool {
        (self.insert_before_this)(self, parent, child)
    }
}
