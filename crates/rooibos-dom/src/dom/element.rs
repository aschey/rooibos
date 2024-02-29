use ratatui::layout::Constraint;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use super::mount_child;
use crate::{IntoView, MountKind, View};

pub struct Element {
    inner: DomNode,
}

impl Element {
    pub fn child(self, child: impl IntoView) -> Self {
        let child = child.into_view();
        mount_child(MountKind::Append(&self.inner), &child);
        self
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

impl IntoView for Element {
    fn into_view(self) -> View {
        View::DomNode(self.inner)
    }
}

pub fn row() -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::row()),
    }
}

pub fn col() -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::col()),
    }
}

pub fn overlay() -> Element {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::overlay()),
    }
}
