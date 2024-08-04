use std::any::type_name;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

use next_tuple::NextTuple;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use reactive_graph::effect::RenderEffect;
use reactive_graph::wrappers::read::MaybeSignal;
use tachys::prelude::*;
use tachys::reactive_graph::RenderEffectState;
use terminput::{KeyEvent, MouseEvent};

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use super::{AsDomNode, Constraint, Focusable, Property};
use crate::widgets::WidgetRole;
use crate::{
    next_node_id, refresh_dom, BlurEvent, Constrainable, EventData, FocusEvent, Role, RooibosDom,
};

pub(crate) type DomWidgetFn = Box<dyn FnMut(Rect, &mut Buffer)>;

#[derive(Clone)]
pub struct DomWidget<P> {
    inner: DomNode,
    properties: P,
}

pub trait WidgetProperty: Property {}

impl WidgetProperty for () {}
impl WidgetProperty for Constraint {}
impl WidgetProperty for Focusable {}

#[derive(Clone)]
pub struct DomWidgetNode {
    f: Rc<dyn Fn() -> DomWidgetFn>,
    rc_f: Rc<RefCell<DomWidgetFn>>,
    id: u32,
    pub(crate) widget_type: String,
    pub(crate) role: Option<Role>,
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
    pub fn new<T: 'static, F1: Fn() -> F2 + 'static, F2: FnMut(Rect, &mut Buffer) + 'static>(
        f: F1,
    ) -> Self {
        let widget_type = type_name::<T>();
        let role = T::widget_role();
        let id = next_node_id();
        let rc_f: Rc<RefCell<DomWidgetFn>> = Rc::new(RefCell::new(Box::new(|_, _| {})));

        Self {
            id,
            role,
            rc_f,
            f: Rc::new(move || Box::new((f)())),
            widget_type: widget_type.into(),
        }
    }

    pub(crate) fn render(&self, rect: Rect, buf: &mut Buffer) {
        (*self.rc_f).borrow_mut()(rect, buf);
    }
}

impl Render<RooibosDom> for DomWidgetNode {
    type State = RenderEffectState<()>;

    fn build(self) -> Self::State {
        RenderEffect::new({
            let f = self.f.clone();
            let rc_f = self.rc_f.clone();
            move |_| {
                (*rc_f.borrow_mut()) = (f)();
                refresh_dom();
            }
        })
        .into()
    }

    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        *state = new;
    }
}

impl DomWidget<()> {
    pub fn new<T: 'static, F1: Fn() -> F2 + 'static, F2: FnMut(Rect, &mut Buffer) + 'static>(
        f: F1,
    ) -> Self {
        let dom_widget_node = DomWidgetNode::new::<T, _, _>(f);
        let inner = DomNode::from_fragment(DocumentFragment::widget(dom_widget_node));
        Self {
            inner,
            properties: (),
        }
    }
}

impl<P> DomWidget<P> {
    pub fn new_with_properties<
        T: 'static,
        F1: Fn() -> F2 + 'static,
        F2: FnMut(Rect, &mut Buffer) + 'static,
    >(
        props: P,
        f: F1,
    ) -> Self {
        let dom_widget_node = DomWidgetNode::new::<T, _, _>(f);
        let inner = DomNode::from_fragment(DocumentFragment::widget(dom_widget_node));
        Self {
            inner,
            properties: props,
        }
    }

    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn class(self, class: impl Into<String>) -> Self {
        self.inner.set_class(class);
        self
    }
}

impl<P> DomWidget<P>
where
    P: NextTuple,
{
    pub fn focusable<S>(self, focusable: S) -> DomWidget<P::Output<Focusable>>
    where
        S: Into<MaybeSignal<bool>>,
    {
        DomWidget {
            inner: self.inner,
            properties: self.properties.next_tuple(Focusable(focusable.into())),
        }
    }

    pub fn on_key_down<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_key_down(handler));

        this
    }

    pub fn on_paste<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(String, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_paste(handler));

        this
    }

    pub fn on_key_up<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(KeyEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_key_up(handler));
        this
    }

    pub fn on_focus<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_focus(handler));
        this
    }

    pub fn on_blur<F>(self, handler: F) -> DomWidget<P::Output<Focusable>>
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        let this = self.focusable(true);
        this.inner
            .update_event_handlers(|event_handlers| event_handlers.on_blur(handler));
        this
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

impl<P> Constrainable for DomWidget<P>
where
    P: NextTuple,
{
    type Output = DomWidget<P::Output<super::Constraint>>;

    fn constraint<S>(self, constraint: S) -> Self::Output
    where
        S: Into<MaybeSignal<ratatui::layout::Constraint>>,
    {
        DomWidget {
            inner: self.inner,
            properties: self
                .properties
                .next_tuple(super::Constraint(constraint.into())),
        }
    }
}

pub struct DomWidgetState<P>
where
    P: WidgetProperty,
{
    node: <DomNode as Render<RooibosDom>>::State,
    prop_state: <P as Property>::State,
}

impl<P> AsDomNode for DomWidgetState<P>
where
    P: WidgetProperty,
{
    fn as_dom_node(&self) -> &DomNode {
        self.node.as_dom_node()
    }
}

impl<P> Mountable<RooibosDom> for DomWidgetState<P>
where
    P: WidgetProperty,
{
    fn unmount(&mut self) {
        self.node.unmount();
    }

    fn mount(
        &mut self,
        parent: &<RooibosDom as Renderer>::Element,
        marker: Option<&<RooibosDom as Renderer>::Node>,
    ) {
        self.node.mount(parent, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<RooibosDom>) -> bool {
        self.node.insert_before_this(child)
    }
}

impl<P> Render<RooibosDom> for DomWidget<P>
where
    P: WidgetProperty,
{
    type State = DomWidgetState<P>;

    fn build(self) -> Self::State {
        let inner_state = self.inner.build();
        let prop_state = self.properties.build(&inner_state.0);
        DomWidgetState {
            node: inner_state,
            prop_state,
        }
    }

    fn rebuild(mut self, state: &mut Self::State) {
        if self.inner == state.node.0 {
            self.inner.rebuild(&mut state.node);
            self.properties
                .rebuild(&state.node.0, &mut state.prop_state);
        } else {
            self.inner.replace_node(&state.node.0);
            *state = self.build();
            refresh_dom();
        }
    }
}

macro_rules! impl_widget_property_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> WidgetProperty for ($($ty,)*)
            where $($ty: WidgetProperty,)*
        { }
    }
}

impl_widget_property_for_tuples!(A);
impl_widget_property_for_tuples!(A, B);
impl_widget_property_for_tuples!(A, B, C);
impl_widget_property_for_tuples!(A, B, C, D);
impl_widget_property_for_tuples!(A, B, C, D, E);
impl_widget_property_for_tuples!(A, B, C, D, E, F);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_widget_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_widget_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
