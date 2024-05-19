use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use next_tuple::NextTuple;
use ratatui::layout::{Constraint, Flex};
use ratatui::widgets::Block;
use reactive_graph::effect::RenderEffect;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::MaybeSignal;
use tachys::prelude::*;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use crate::{notify, ChildState, Constrainable, EventData, RenderAny, RooibosDom};

#[derive(Debug)]
pub struct Element<C> {
    inner: DomNode,
    children: C,
}

impl<C> Element<C>
where
    C: NextTuple,
{
    pub fn child<T>(self, child: T) -> Element<C::Output<T>> {
        Element {
            inner: self.inner,
            children: self.children.next_tuple(child),
        }
    }
}

impl<C> Element<C> {
    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn margin<S>(self, margin: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        let layout_props = self.inner.layout_props();
        let effect = RenderEffect::new({
            let margin = margin.into();
            move |_| {
                layout_props.borrow_mut().margin = margin.get();
                notify();
            }
        });
        self.inner.add_data(effect);
        self
    }

    pub fn flex<S>(self, flex: S) -> Self
    where
        S: Into<MaybeSignal<Flex>>,
    {
        let layout_props = self.inner.layout_props();
        let effect = RenderEffect::new({
            let flex = flex.into();
            move |_| {
                layout_props.borrow_mut().flex = flex.get();
                notify();
            }
        });
        self.inner.add_data(effect);
        self
    }

    pub fn spacing<S>(self, spacing: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        let layout_props = self.inner.layout_props();
        let effect = RenderEffect::new({
            let spacing = spacing.into();
            move |_| {
                layout_props.borrow_mut().spacing = spacing.get();
                notify();
            }
        });
        self.inner.add_data(effect);
        self
    }

    pub fn block<S>(self, block: S) -> Self
    where
        S: Into<MaybeSignal<Block<'static>>>,
    {
        let layout_props = self.inner.layout_props();
        let effect = RenderEffect::new({
            let block = block.into();
            move |_| {
                layout_props.borrow_mut().block = Some(block.get());

                notify();
            }
        });
        self.inner.add_data(effect);
        self
    }

    pub fn focusable<S>(self, focusable: S) -> Self
    where
        S: Into<MaybeSignal<bool>>,
    {
        let focusable_rc = Rc::new(RefCell::new(false));

        let effect = RenderEffect::new({
            let focusable = focusable.into();
            let focusable_rc = focusable_rc.clone();
            move |_| {
                *focusable_rc.borrow_mut() = focusable.get();
                notify();
            }
        });
        self.inner.set_focusable(focusable_rc);
        self.inner.add_data(effect);
        self
    }

    pub fn on_focus<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_focus(handler));
        self
    }

    pub fn on_blur<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_blur(handler));
        self
    }
}

impl<C> Constrainable for Element<C> {
    fn constraint<S>(self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        let constraint_rc = Rc::new(RefCell::new(Constraint::default()));

        let effect = RenderEffect::new({
            let constraint = constraint.into();
            let constraint_rc = constraint_rc.clone();
            move |_| {
                *constraint_rc.borrow_mut() = constraint.get();
                notify();
            }
        });
        self.inner.set_constraint(constraint_rc);
        self.inner.add_data(effect);
        self
    }
}

#[macro_export]
macro_rules! row {
    () => (
        $crate::row(())
    );
    ($x:expr) => (
        $crate::row(($x,))
    );
    ($($x:expr),+ $(,)?) => (
        $crate::row(($($x),+))
    );
}

#[macro_export]
macro_rules! col {
    () => (
        $crate::col(())
    );
    ($x:expr) => (
        $crate::col(($x,))
    );
    ($($x:expr),+ $(,)?) => (
        $crate::col(($($x),+))
    );
}

#[macro_export]
macro_rules! overlay {
    () => (
        $crate::overlay(())
    );
    ($x:expr) => (
        $crate::overlay(($x,))
    );
    ($($x:expr),+ $(,)?) => (
        $crate::overlay(($($x),+))
    );
}

pub fn row<C>(children: C) -> Element<C> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::row()),
        children,
    }
}

pub fn col<C>(children: C) -> Element<C> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::col()),
        children,
    }
}

pub fn overlay<C>(children: C) -> Element<C> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::overlay()),
        children,
    }
}

impl<Children> Render<RooibosDom> for Element<Children>
where
    Children: RenderAny + 'static,
{
    type State = DomNode;

    fn build(self) -> Self::State {
        let mut children_state = self.children.build();
        children_state.mount(&self.inner, None);
        // Store children output to prevent drop effects from occurring
        self.inner.set_child_state(ChildState {
            mountable: Box::new(children_state),
            parent: self.inner.clone(),
        });
        self.inner
    }

    fn rebuild(self, state: &mut Self::State) {
        let mut child_state = state.take_child_state();

        let child_mountable = child_state
            .mountable
            .as_any()
            .downcast_mut::<Children::State>();

        if let Some(s) = child_mountable {
            self.children.rebuild(s);
        } else if &self.inner != state {
            child_state.mountable.unmount();
            let mut new = self.build();
            new.mount(&child_state.parent, None);
        }

        state.set_child_state(child_state);
    }
}
