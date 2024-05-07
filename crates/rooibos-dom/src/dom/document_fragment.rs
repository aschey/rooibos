use std::cell::RefCell;
use std::rc::Rc;

use derivative::Derivative;
use ratatui::layout::{Constraint, Direction, Flex};

use super::dom_node::{NodeId, NodeType};
use super::dom_widget::DomWidget;
use super::KeyEventFn;
use crate::LayoutProps;

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
    pub(crate) on_key_down: Option<KeyEventFn>,
}

impl DocumentFragment {
    pub(crate) fn widget(widget: DomWidget) -> Self {
        Self {
            name: widget.widget_type.clone(),
            constraint: widget.constraint,
            node_type: NodeType::Widget(widget),
            flex: Flex::default(),
            focusable: false,
            id: None,

            on_key_down: None,
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
            on_key_down: None,
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
            on_key_down: None,
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
            on_key_down: None,
        }
    }

    pub(crate) fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }

    pub(crate) fn id(mut self, id: Option<NodeId>) -> Self {
        self.id = id;
        self
    }

    pub(crate) fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub(crate) fn on_key_down(mut self, handler: Option<KeyEventFn>) -> Self {
        self.on_key_down = handler;
        self
    }
}
