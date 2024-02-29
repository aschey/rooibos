use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use ratatui::layout::{Constraint, Rect};
use ratatui::Frame;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use crate::{IntoView, Mountable, View};

#[derive(Clone)]
pub struct DomWidget {
    f: Rc<RefCell<dyn FnMut(&mut Frame, Rect)>>,
    id: u32,
    pub(crate) widget_type: String,
    pub(crate) constraint: Constraint,
    dom_id: Option<NodeId>,
}

impl Debug for DomWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}/>", self.widget_type)
    }
}

impl DomWidget {
    pub fn new<F: FnMut(&mut Frame, Rect) + 'static>(
        id: u32,
        widget_type: impl Into<String>,
        f: F,
    ) -> Self {
        Self {
            widget_type: widget_type.into(),
            id,
            f: Rc::new(RefCell::new(f)),
            constraint: Constraint::default(),
            dom_id: None,
        }
    }

    pub(crate) fn render(&self, frame: &mut Frame, rect: Rect) {
        (*self.f).borrow_mut()(frame, rect)
    }

    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn id(mut self, id: impl Into<NodeId>) -> Self {
        self.dom_id = Some(id.into());
        self
    }
}

impl IntoView for DomWidget {
    fn into_view(self) -> View {
        View::DomWidget(self)
    }
}

impl Mountable for DomWidget {
    fn get_mountable_node(&self) -> DomNode {
        DomNode::from_fragment(
            DocumentFragment::widget(self.clone())
                .constraint(self.constraint)
                .id(self.dom_id.clone()),
        )
    }
}

impl PartialEq for DomWidget {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DomWidget {}
