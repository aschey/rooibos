use std::cell::RefCell;
use std::rc::Rc;

use derivative::Derivative;
use ratatui::layout::{Constraint, Direction, Flex};

use super::dom_node::{NodeId, NodeType};
use crate::{DomWidgetNode, EventHandlers, LayoutProps};

#[derive(Derivative)]
#[derivative(PartialEq, Eq)]
#[derive(Clone)]
pub struct DocumentFragment {
    pub(crate) node_type: NodeType,
    pub(crate) constraint: Constraint,
    pub(crate) id: Option<NodeId>,
    pub(crate) focusable: bool,
    pub(crate) flex: Flex,
    pub(crate) name: String,
    #[derivative(PartialEq = "ignore")]
    pub(crate) event_handlers: EventHandlers,
}

impl DocumentFragment {
    pub(crate) fn widget(widget: DomWidgetNode) -> Self {
        Self {
            name: widget.widget_type.clone(),
            constraint: Constraint::default(),
            node_type: NodeType::Widget(widget),
            flex: Flex::default(),
            focusable: false,
            id: None,
            event_handlers: Default::default(),
        }
    }

    pub(crate) fn row() -> Self {
        Self {
            node_type: NodeType::Layout(Rc::new(RefCell::new(LayoutProps {
                direction: Direction::Horizontal,
                ..Default::default()
            }))),
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "row".to_string(),
            focusable: false,
            id: None,
            event_handlers: Default::default(),
        }
    }

    pub(crate) fn col() -> Self {
        Self {
            node_type: NodeType::Layout(Rc::new(RefCell::new(LayoutProps {
                direction: Direction::Vertical,
                ..Default::default()
            }))),
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "col".to_string(),
            focusable: false,
            id: None,
            event_handlers: Default::default(),
        }
    }

    pub(crate) fn overlay() -> Self {
        Self {
            node_type: NodeType::Overlay,
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "overlay".to_string(),
            focusable: false,
            id: None,
            event_handlers: Default::default(),
        }
    }
}
