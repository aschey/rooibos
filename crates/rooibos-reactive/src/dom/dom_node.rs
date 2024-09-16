use core::fmt::Debug;
use std::any::Any;
use std::ops::{Deref, DerefMut};

use reactive_graph::effect::RenderEffect;
use rooibos_dom2::{unmount_child, AsDomNode};
use tachys::renderer::Renderer;
use tachys::view::{Mountable, Render};

use super::{with_nodes, RooibosDom};
use crate::DomWidgetNode;

#[derive(Debug)]
pub struct NodeType(rooibos_dom2::NodeType);

impl Render<RooibosDom> for NodeType {
    type State = Option<RenderEffect<()>>;

    fn build(self) -> Self::State {
        match self.0 {
            rooibos_dom2::NodeType::Layout => None,
            rooibos_dom2::NodeType::Widget(node) => Some(DomWidgetNode(node).build()),
            rooibos_dom2::NodeType::Placeholder => None,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        match self.0 {
            rooibos_dom2::NodeType::Layout => {}
            rooibos_dom2::NodeType::Widget(node) => {
                if let Some(s) = state.as_mut() {
                    DomWidgetNode(node).rebuild(s)
                }
            }
            rooibos_dom2::NodeType::Placeholder => {}
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
pub struct DomNode(pub(crate) rooibos_dom2::DomNode);

impl AsDomNode for DomNode {
    fn as_dom_node(&self) -> &rooibos_dom2::DomNode {
        self.0.as_dom_node()
    }
}

impl Deref for DomNode {
    type Target = rooibos_dom2::DomNode;

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
        let state = with_nodes(|n| NodeType(n.node_type(self.0.get_key()).clone()).build());
        (self, state)
    }

    fn rebuild(self, (_node, ref mut node_type_state): &mut Self::State) {
        with_nodes(|n| NodeType(n.node_type(self.0.get_key()).clone()).rebuild(node_type_state));
    }
}
