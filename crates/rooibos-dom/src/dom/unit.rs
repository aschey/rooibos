use std::fmt;

use crate::{DomNode, IntoView, Mountable, View};

#[derive(Clone, Copy, Debug)]
pub struct Unit;

impl IntoView for Unit {
    fn into_view(self) -> View {
        let component = UnitRepr::default();

        View::Unit(component)
    }
}

impl IntoView for () {
    fn into_view(self) -> View {
        Unit.into_view()
    }
}

/// The internal representation of the [`Unit`] core-component.
#[derive(Clone, PartialEq, Eq)]
pub struct UnitRepr {
    node: DomNode,
}

impl fmt::Debug for UnitRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<() />")
    }
}

impl Default for UnitRepr {
    fn default() -> Self {
        Self {
            node: DomNode::transparent("()"),
        }
    }
}

impl Mountable for UnitRepr {
    fn get_mountable_node(&self) -> DomNode {
        self.node.clone()
    }
}
