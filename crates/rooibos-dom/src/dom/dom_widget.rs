use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use ratatui::layout::{Constraint, Rect};
use ratatui::Frame;
use reactive_graph::computed::Memo;
use reactive_graph::effect::RenderEffect;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::MaybeSignal;
use tachys::prelude::*;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use crate::{
    next_node_id, refresh_dom, Constrainable, EventData, KeyEvent, MouseEvent, RooibosDom,
};

pub(crate) type DomWidgetFn = Box<dyn FnMut(&mut Frame, Rect)>;

#[derive(Clone)]
pub struct DomWidget {
    inner: DomNode,
}

#[derive(Clone)]
pub(crate) struct DomWidgetNode {
    f: Rc<RefCell<DomWidgetFn>>,
    id: u32,
    pub(crate) widget_type: String,
    _effect: Rc<RenderEffect<()>>,
}

impl PartialEq for DomWidgetNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DomWidgetNode {}

impl Debug for DomWidgetNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}/>", self.widget_type)
    }
}

impl DomWidgetNode {
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
                refresh_dom();
            }
        });
        Self {
            id,
            f: rc_f,
            widget_type: widget_type.into(),
            _effect: Rc::new(effect),
        }
    }

    pub(crate) fn render(&self, frame: &mut Frame, rect: Rect) {
        (*self.f).borrow_mut()(frame, rect);
    }
}

impl DomWidget {
    pub fn new<F1: Fn() -> F2 + 'static, F2: FnMut(&mut Frame, Rect) + 'static>(
        widget_type: impl Into<String>,
        f: F1,
    ) -> Self {
        let dom_widget_node = DomWidgetNode::new(widget_type, f);
        let inner = DomNode::from_fragment(DocumentFragment::widget(dom_widget_node));
        Self { inner }
    }

    pub(crate) fn inner(&self) -> DomNode {
        self.inner.clone()
    }

    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn focusable<S>(self, focusable: S) -> Self
    where
        S: Into<MaybeSignal<bool>>,
    {
        let focusable_rc = Rc::new(RefCell::new(false));

        let effect = RenderEffect::new({
            let focusable_rc = focusable_rc.clone();
            let focusable = focusable.into();
            let focusable = Memo::new(move |_| focusable.get());
            move |_| {
                *focusable_rc.borrow_mut() = focusable.get();
                refresh_dom();
            }
        });
        self.inner.set_focusable(focusable_rc);
        self.inner.add_data(effect);
        self
    }

    pub fn on_key_down<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        self = self.focusable(true);
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_key_down(handler));

        self
    }

    pub fn on_key_up<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        self = self.focusable(true);
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_key_up(handler));
        self
    }

    pub fn on_focus<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self = self.focusable(true);
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_focus(handler));
        self
    }

    pub fn on_blur<F>(mut self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self = self.focusable(true);
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_blur(handler));
        self
    }

    pub fn on_click<F>(self, handler: F) -> Self
    where
        F: FnMut(MouseEvent, EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_click(handler));
        self
    }

    pub fn on_mouse_enter<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_mouse_enter(handler));
        self
    }

    pub fn on_mouse_leave<F>(self, handler: F) -> Self
    where
        F: FnMut(EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_mouse_leave(handler));
        self
    }

    pub fn on_size_change<F>(self, handler: F) -> Self
    where
        F: FnMut(Rect) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_size_change(handler));
        self
    }
}

impl Constrainable for DomWidget {
    fn constraint<S>(self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>,
    {
        let constraint_rc = Rc::new(RefCell::new(Constraint::default()));

        let effect = RenderEffect::new({
            let constraint_rc = constraint_rc.clone();
            let constraint = constraint.into();
            let constraint = Memo::new(move |_| constraint.get());
            move |_| {
                *constraint_rc.borrow_mut() = constraint.get();
                refresh_dom();
            }
        });
        self.inner.set_constraint(constraint_rc);
        self.inner.add_data(effect);
        self
    }
}

impl Render<RooibosDom> for DomWidget {
    type State = DomNode;

    fn build(self) -> Self::State {
        self.inner
    }

    fn rebuild(mut self, state: &mut Self::State) {
        if &self.inner != state {
            self.inner.replace_node(state);
            refresh_dom();
        }
    }
}
