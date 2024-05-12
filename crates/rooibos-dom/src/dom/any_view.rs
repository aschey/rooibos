use std::any::{Any, TypeId};
use std::fmt::Debug;

use tachys::view::{Mountable, Render};

use crate::{DomNode, RenderAny, RooibosDom};

#[derive(Debug)]
pub struct AnyView {
    type_id: TypeId,
    value: Box<dyn Any + Send>,
    build: fn(Box<dyn Any>) -> AnyViewState,
    rebuild: fn(TypeId, Box<dyn Any>, &mut AnyViewState),
}

pub struct AnyViewState {
    type_id: TypeId,
    parent: Option<DomNode>,
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
    T: RenderAny,
    T::State: 'static,
{
    let state = state
        .downcast_mut::<T::State>()
        .expect("AnyViewState::as_mountable couldn't downcast state");
    state.mount(parent, marker)
}

fn unmount_any<T>(state: &mut dyn Any)
where
    T: RenderAny,
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
    T: RenderAny,
    T::State: 'static,
{
    let state = state
        .downcast_ref::<T::State>()
        .expect("AnyViewState::opening_node couldn't downcast state");
    state.insert_before_this(parent, child)
}

impl<T> IntoAny for T
where
    T: Send,
    T: RenderAny + 'static,
    T::State: 'static,
{
    fn into_any(self) -> AnyView {
        let value = Box::new(self) as Box<dyn Any + Send>;

        let build = |value: Box<dyn Any>| {
            let value = value
                .downcast::<T>()
                .expect("AnyView::build couldn't downcast");
            let state = Box::new(value.build());

            AnyViewState {
                type_id: TypeId::of::<T>(),
                state,
                parent: None,
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
                let mut new = value.into_any().build();

                state.unmount();
                if let Some(parent) = &state.parent {
                    new.mount(parent, None);
                }

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

    fn build(self) -> Self::State {
        (self.build)(self.value)
    }

    fn rebuild(self, state: &mut Self::State) {
        (self.rebuild)(self.type_id, self.value, state)
    }
}

impl Mountable<RooibosDom> for AnyViewState {
    fn unmount(&mut self) {
        (self.unmount)(&mut *self.state)
    }

    fn mount(&mut self, parent: &DomNode, marker: Option<&DomNode>) {
        self.parent = Some(parent.clone());
        (self.mount)(&mut *self.state, parent, marker)
    }

    fn insert_before_this(&self, parent: &DomNode, child: &mut dyn Mountable<RooibosDom>) -> bool {
        (self.insert_before_this)(self, parent, child)
    }
}
