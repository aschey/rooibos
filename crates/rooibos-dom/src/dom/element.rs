use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use next_tuple::NextTuple;
use ratatui::layout::{Constraint, Flex};
use reactive_graph::effect::RenderEffect;
use tachys::prelude::*;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use crate::{notify, RenderAny, RooibosDom};

#[derive(Debug)]
pub struct Element<Children> {
    inner: DomNode,
    children: Children,
}

pub trait ToProperty<T> {
    fn to_property(&self) -> T;
}

impl<F, T> ToProperty<T> for F
where
    F: Fn() -> T,
{
    fn to_property(&self) -> T {
        self()
    }
}

impl ToProperty<Constraint> for Constraint {
    fn to_property(&self) -> Constraint {
        *self
    }
}

impl<Children> Element<Children>
where
    Children: NextTuple,
{
    pub fn child<T>(self, child: T) -> Element<Children::Output<T>> {
        Element {
            inner: self.inner,
            children: self.children.next_tuple(child),
        }
    }

    pub fn constraint<T>(self, constraint: T) -> Self
    where
        T: ToProperty<Constraint> + 'static,
    {
        let constraint_rc = Rc::new(RefCell::new(Constraint::default()));
        let effect = RenderEffect::new({
            let constraint_rc = constraint_rc.clone();
            move |_| {
                *constraint_rc.borrow_mut() = constraint.to_property();
                notify();
            }
        });
        self.inner.set_constraint(constraint_rc);
        self.inner.add_data(effect);
        self
    }

    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn margin<T>(self, margin: T) -> Self
    where
        T: ToProperty<u16> + 'static,
    {
        let layout_props = self.inner.layout_props();
        let effect = RenderEffect::new({
            move |_| {
                layout_props.borrow_mut().margin = margin.to_property();
                notify();
            }
        });
        self.inner.add_data(effect);
        self
    }

    pub fn flex<T>(self, flex: T) -> Self
    where
        T: ToProperty<Flex> + 'static,
    {
        let layout_props = self.inner.layout_props();
        let effect = RenderEffect::new({
            move |_| {
                layout_props.borrow_mut().flex = flex.to_property();
                notify();
            }
        });
        self.inner.add_data(effect);
        self
    }

    pub fn spacing<T>(self, spacing: T) -> Self
    where
        T: ToProperty<u16> + 'static,
    {
        let layout_props = self.inner.layout_props();
        let effect = RenderEffect::new({
            move |_| {
                layout_props.borrow_mut().spacing = spacing.to_property();
                notify();
            }
        });
        self.inner.add_data(effect);
        self
    }
}

pub fn row() -> Element<()> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::row()),
        children: (),
    }
}

pub fn col() -> Element<()> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::col()),
        children: (),
    }
}

pub fn overlay() -> Element<()> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::overlay()),
        children: (),
    }
}

impl<Children> Render<RooibosDom> for Element<Children>
where
    Children: RenderAny + 'static,
{
    type State = DomNode;

    fn build(self) -> Self::State {
        let mut children = self.children.build();
        children.mount(&self.inner, None);
        // Store children output to prevent drop effects from occurring
        self.inner.add_data(children);
        self.inner
    }

    fn rebuild(self, _state: &mut Self::State) {}
}
