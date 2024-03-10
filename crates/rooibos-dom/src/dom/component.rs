use std::cell::OnceCell;
use std::rc::Rc;

use rooibos_reactive::untrack_with_diagnostics;

use super::document_fragment::DocumentFragment;
use super::dom_node::DomNode;
use super::mount_child;
use crate::{next_node_id, IntoView, MountKind, Mountable, View};

#[derive(Clone, PartialEq, Eq)]
pub struct ComponentRepr {
    document_fragment: DomNode,
    mounted: Rc<OnceCell<DomNode>>,
    children: Vec<View>,
    id: u32,
}

impl ComponentRepr {
    pub fn new_with_id(name: impl Into<String>, id: u32, children: Vec<View>) -> Self {
        let name = name.into();
        let document_fragment = DocumentFragment::transparent(name.clone());

        Self {
            document_fragment: DomNode::from_fragment(document_fragment),
            mounted: Default::default(),
            children,
            id,
        }
    }

    pub(crate) fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
    }
}

impl Mountable for ComponentRepr {
    fn get_mountable_node(&self) -> DomNode {
        if let Some(mounted) = self.mounted.get() {
            mounted.clone()
        } else {
            for child in &self.children {
                mount_child(MountKind::Append(&self.document_fragment), child);
            }
            let node = self.document_fragment.clone();
            self.mounted.set(node.clone()).unwrap();
            node
        }
    }
}

impl IntoView for ComponentRepr {
    fn into_view(self) -> View {
        View::Component(self)
    }
}

pub struct Component<F, V>
where
    F: FnOnce() -> V,
    V: IntoView,
{
    id: u32,
    name: String,
    children_fn: F,
}

impl<F, V> Component<F, V>
where
    F: FnOnce() -> V,
    V: IntoView,
{
    /// Creates a new component.
    pub fn new(name: impl Into<String>, f: F) -> Self {
        Self {
            id: next_node_id(),
            name: name.into(),
            children_fn: f,
        }
    }
}

impl<F, V> IntoView for Component<F, V>
where
    F: FnOnce() -> V,
    V: IntoView,
{
    fn into_view(self) -> View {
        let Self {
            id,
            name,
            children_fn,
        } = self;

        // disposed automatically when the parent scope is disposed
        let child = untrack_with_diagnostics(|| children_fn().into_view());
        let repr = ComponentRepr::new_with_id(name, id, vec![child]);

        repr.into_view()
    }
}
