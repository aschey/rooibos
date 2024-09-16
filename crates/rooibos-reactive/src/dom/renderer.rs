use ratatui::widgets::WidgetRef;
use rooibos_dom::{clear_children, refresh_dom, unmount_child, AsDomNode};
use tachys::prelude::Renderer;
use tachys::renderer::CastFrom;
use tachys::view::Render as _;
use tracing::info;

use super::{text, DomNode};
use crate::wgt;

pub trait RenderAny: tachys::view::Render<RooibosDom> {}

impl<T> RenderAny for T where T: tachys::view::Render<RooibosDom> {}

pub trait Render: tachys::view::Render<RooibosDom, State = Self::DomState> {
    type DomState: AsDomNode;
}

impl<T, S> Render for T
where
    T: tachys::view::Render<RooibosDom, State = S>,
    S: AsDomNode,
{
    type DomState = S;
}

#[derive(Debug)]
pub struct RooibosDom;

impl Renderer for RooibosDom {
    type Node = DomNode;

    type Element = DomNode;

    type Text = DomNode;

    type Placeholder = DomNode;

    fn intern(text: &str) -> &str {
        #[cfg(target_arch = "wasm32")]
        return wasm_bindgen::intern(text);
        #[cfg(not(target_arch = "wasm32"))]
        return text;
    }

    fn create_text_node(text: &str) -> Self::Text {
        let text = text.to_owned();
        DomNode(wgt!(text!(text.clone())).build().as_dom_node().clone())
    }

    fn create_placeholder() -> Self::Placeholder {
        let placeholder = DomNode(rooibos_dom::DomNode::placeholder());
        DomNode(placeholder.build().as_dom_node().clone())
    }

    fn set_text(node: &Self::Text, text: &str) {
        let text = text.to_string();
        node.replace_widget(rooibos_dom::DomWidgetNode::new::<String, _, _>(move || {
            let text = text.clone();
            move |rect, buf| {
                text.render_ref(rect, buf);
            }
        }));
        node.clone().build();
    }

    fn set_attribute(_node: &Self::Element, _name: &str, _value: &str) {
        unimplemented!("set attribute not supported")
    }

    fn remove_attribute(_node: &Self::Element, _name: &str) {
        unimplemented!("remove attribute not supported")
    }

    fn insert_node(parent: &Self::Element, new_child: &Self::Node, marker: Option<&Self::Node>) {
        parent.insert_before(new_child, marker);
        refresh_dom();
    }

    fn remove_node(_parent: &Self::Element, child: &Self::Node) -> Option<Self::Node> {
        unmount_child(child.get_key(), true);
        Some(child.clone())
    }

    fn clear_children(parent: &Self::Element) {
        clear_children(parent.get_key())
    }

    fn remove(node: &Self::Node) {
        unmount_child(node.get_key(), true);
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        node.get_parent().map(DomNode)
    }

    fn first_child(node: &Self::Node) -> Option<Self::Node> {
        node.get_first_child().map(DomNode)
    }

    fn next_sibling(node: &Self::Node) -> Option<Self::Node> {
        node.get_next_sibling().map(DomNode)
    }

    fn log_node(node: &Self::Node) {
        info!("{:?}", node);
    }
}

impl CastFrom<DomNode> for DomNode {
    fn cast_from(source: DomNode) -> Option<Self> {
        Some(source)
    }
}

impl AsRef<DomNode> for DomNode {
    fn as_ref(&self) -> &DomNode {
        self
    }
}
