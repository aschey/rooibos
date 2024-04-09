use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;

use next_tuple::TupleBuilder;
use ratatui::layout::Constraint;
use tachys::prelude::*;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use super::mount_child;
use crate::{MountKind, RooibosDom};

#[derive(Debug)]
pub struct Element<Children> {
    inner: DomNode,
    children: Children,
}

impl<Children> Element<Children>
where
    Children: TupleBuilder,
{
    pub fn child<T>(self, child: T) -> Element<Children::Output<T>> {
        Element {
            inner: self.inner,
            children: self.children.next_tuple(child),
        }
    }

    pub fn constraint(self, constraint: Constraint) -> Self {
        self.inner.set_constraint(constraint);
        self
    }

    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn margin(self, margin: u16) -> Self {
        self.inner.set_margin(margin);
        self
    }
}

// impl<Children> ToDomNode for Element<Children> {
//     fn to_dom_node(&self) -> DomNode {
//         self.inner.clone()
//     }
// }

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
    Children: Render<RooibosDom> + 'static,
{
    type State = DomNode;

    type FallibleState = DomNode;

    type AsyncOutput = ();

    fn build(self) -> Self::State {
        let mut children = self.children.build();
        children.mount(&self.inner, None);
        // Store children output to prevent drop effects from occurring
        self.inner.set_data(children);
        self.inner
    }

    fn rebuild(self, state: &mut Self::State) {
        // todo!()
        // self.children.rebuild(state);
    }

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
