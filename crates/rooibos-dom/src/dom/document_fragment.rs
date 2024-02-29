use ratatui::layout::{Constraint, Direction, Flex};

use super::dom_node::{NodeId, NodeType};
use super::dom_widget::DomWidget;

#[derive(Clone, PartialEq, Eq)]
pub struct DocumentFragment {
    pub(crate) node_type: NodeType,
    pub(crate) constraint: Constraint,
    pub(crate) id: Option<NodeId>,
    pub(crate) flex: Flex,
    pub(crate) name: String,
}

impl DocumentFragment {
    pub(crate) fn widget(widget: DomWidget) -> Self {
        Self {
            name: widget.widget_type.clone(),
            constraint: widget.constraint,
            node_type: NodeType::Widget(widget),
            flex: Flex::default(),
            id: None,
        }
    }

    pub(crate) fn row() -> Self {
        Self {
            node_type: NodeType::Layout {
                direction: Direction::Horizontal,
                flex: Flex::default(),
                margin: 0,
                spacing: 0,
            },
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "row".to_string(),
            id: None,
        }
    }

    pub(crate) fn col() -> Self {
        Self {
            node_type: NodeType::Layout {
                direction: Direction::Vertical,
                flex: Flex::default(),
                margin: 0,
                spacing: 0,
            },
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "col".to_string(),
            id: None,
        }
    }

    pub(crate) fn overlay() -> Self {
        Self {
            node_type: NodeType::Overlay,
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: "overlay".to_string(),
            id: None,
        }
    }

    pub(crate) fn transparent(name: impl Into<String>) -> Self {
        Self {
            node_type: NodeType::Transparent,
            constraint: Constraint::default(),
            flex: Flex::default(),
            name: name.into(),
            id: None,
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
}
