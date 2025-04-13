use core::fmt::Debug;
use std::any::Any;
use std::fmt;
use std::ops::{Deref, DerefMut};

use reactive_graph::effect::RenderEffect;
use reactive_graph::owner::StoredValue;
use reactive_graph::traits::{GetValue, WithValue};
use rooibos_dom::{AsDomNode, unmount_child};
use tachys::renderer::Renderer;
use tachys::view::{Mountable, Render};

use super::{RooibosDom, with_nodes};
use crate::dom::DomWidgetNode;

#[derive(Clone, Debug, Copy)]
pub struct NodeId(pub(crate) StoredValue<rooibos_dom::NodeId>);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.with_value(|v| fmt::Display::fmt(v, f))
    }
}

impl NodeId {
    pub fn new_auto() -> Self {
        Self(StoredValue::new(rooibos_dom::NodeId::new_auto()))
    }

    pub fn new(id: impl Into<String>) -> Self {
        Self(StoredValue::new(rooibos_dom::NodeId::new(id)))
    }
}

impl From<NodeId> for rooibos_dom::NodeId {
    fn from(value: NodeId) -> Self {
        value.0.get_value()
    }
}

impl From<rooibos_dom::NodeId> for NodeId {
    fn from(value: rooibos_dom::NodeId) -> Self {
        Self(StoredValue::new(value))
    }
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        self.0.with_value(|v1| other.0.with_value(|v2| v1 == v2))
    }
}

impl Eq for NodeId {}

#[derive(Debug)]
pub struct NodeType(rooibos_dom::NodeType);

impl Render<RooibosDom> for NodeType {
    type State = Option<RenderEffect<()>>;

    fn build(self) -> Self::State {
        match self.0 {
            rooibos_dom::NodeType::Layout => None,
            rooibos_dom::NodeType::FocusScope(_) => None,
            rooibos_dom::NodeType::Widget(node) => Some(DomWidgetNode(node).build()),
            rooibos_dom::NodeType::Placeholder => None,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        match self.0 {
            rooibos_dom::NodeType::Layout => {}
            rooibos_dom::NodeType::FocusScope(_) => {}
            rooibos_dom::NodeType::Widget(node) => {
                if let Some(s) = state.as_mut() {
                    DomWidgetNode(node).rebuild(s)
                }
            }
            rooibos_dom::NodeType::Placeholder => {}
        }
    }
}

pub trait AnyMountable: Mountable<RooibosDom> + Any {
    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> AnyMountable for T
where
    T: Mountable<RooibosDom> + Any,
{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl Mountable<RooibosDom> for Box<dyn AnyMountable> {
    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        (**self).mount(parent, marker)
    }

    fn unmount(&mut self) {
        (**self).unmount()
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        (**self).insert_before_this(child)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DomNode(pub(crate) rooibos_dom::DomNode);

impl AsDomNode for DomNode {
    fn as_dom_node(&self) -> &rooibos_dom::DomNode {
        self.0.as_dom_node()
    }
}

impl Deref for DomNode {
    type Target = rooibos_dom::DomNode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DomNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Mountable<RooibosDom> for DomNode {
    fn unmount(&mut self) {
        unmount_child(self.0.get_key(), false);
    }

    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        RooibosDom::insert_node(parent, self, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        if let Some(parent) = RooibosDom::get_parent(self) {
            child.mount(&parent, Some(self));
            return true;
        }
        false
    }
}

impl Render<RooibosDom> for DomNode {
    type State = (DomNode, <NodeType as Render<RooibosDom>>::State);

    fn build(self) -> Self::State {
        let node_type = with_nodes(|n| NodeType(n.node_type(self.0.get_key()).clone()));
        let state = node_type.build();
        (self, state)
    }

    fn rebuild(self, &mut (ref _node, ref mut node_type_state): &mut Self::State) {
        let node_type = with_nodes(|n| NodeType(n.node_type(self.0.get_key()).clone()));
        node_type.rebuild(node_type_state);
    }
}
