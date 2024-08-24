use ratatui::widgets::WidgetRef;
use tachys::prelude::Renderer;
use tachys::renderer::CastFrom;
use tachys::view::Render as _;
use tracing::info;

use super::{clear_children, refresh_dom, text, unmount_child, AsDomNode, DomNode, DomWidgetNode};
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
        wgt!(text!(text.clone())).build().as_dom_node().clone()
    }

    fn create_placeholder() -> Self::Placeholder {
        let placeholder = DomNode::placeholder();
        placeholder.build().as_dom_node().clone()
    }

    fn set_text(node: &Self::Text, text: &str) {
        let text = text.to_string();
        node.replace_widget(DomWidgetNode::new::<String, _, _>(move || {
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
        unmount_child(child.key(), true);

        Some(child.clone())
    }

    fn clear_children(parent: &Self::Element) {
        clear_children(parent.key())
    }

    fn remove(node: &Self::Node) {
        unmount_child(node.key(), true);
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        node.get_parent()
    }

    fn first_child(node: &Self::Node) -> Option<Self::Node> {
        node.get_first_child()
    }

    fn next_sibling(node: &Self::Node) -> Option<Self::Node> {
        node.get_next_sibling()
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
