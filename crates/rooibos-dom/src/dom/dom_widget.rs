use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use ratatui::layout::{Constraint, Rect};
use ratatui::Frame;
use reactive_graph::effect::RenderEffect;
use tachys::prelude::*;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use crate::{next_node_id, notify, RooibosDom};

type DomWidgetFn = Box<dyn FnMut(&mut Frame, Rect)>;

#[derive(Clone)]
pub struct DomWidget {
    f: Rc<RefCell<DomWidgetFn>>,
    id: u32,
    pub(crate) widget_type: String,
    pub(crate) constraint: Constraint,
    dom_id: Option<NodeId>,
    _effect: Rc<RenderEffect<()>>,
}

impl Debug for DomWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}/>", self.widget_type)
    }
}

impl DomWidget {
    pub fn new<F1: Fn() -> F2 + 'static, F2: FnMut(&mut Frame, Rect) + 'static>(
        widget_type: impl Into<String>,
        f: F1,
    ) -> Self {
        let id = next_node_id();
        let rc_f: Rc<RefCell<DomWidgetFn>> = Rc::new(RefCell::new(Box::new(|_, _| {})));

        let effect = RenderEffect::new({
            let rc_f = rc_f.clone();
            move |_| {
                (*rc_f.borrow_mut()) = Box::new((f)());
                notify();
            }
        });
        Self {
            widget_type: widget_type.into(),
            id,
            f: rc_f,
            constraint: Constraint::default(),
            dom_id: None,
            _effect: Rc::new(effect),
        }
    }

    pub(crate) fn render(&self, frame: &mut Frame, rect: Rect) {
        (*self.f).borrow_mut()(frame, rect);
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

impl Render<RooibosDom> for DomWidget {
    type State = DomNode;

    fn build(self) -> Self::State {
        DomNode::from_fragment(
            DocumentFragment::widget(self.clone())
                .constraint(self.constraint)
                .id(self.dom_id.clone()),
        )
    }

    fn rebuild(self, _state: &mut Self::State) {
        todo!()
    }
}

impl PartialEq for DomWidget {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DomWidget {}
