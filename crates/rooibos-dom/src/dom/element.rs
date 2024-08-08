use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use next_tuple::NextTuple;
use ratatui::layout::Rect;
use reactive_graph::computed::Memo;
use reactive_graph::effect::RenderEffect;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::MaybeSignal;
use tachys::prelude::*;

use super::document_fragment::DocumentFragment;
use super::dom_node::{DomNode, NodeId};
use super::AsDomNode;
use crate::{
    derive_signal, refresh_dom, BlurEvent, Constrainable, EventData, FocusEvent, RenderAny,
    RooibosDom,
};

pub trait Property {
    type State;

    fn build(self, node: &DomNode) -> Self::State;

    fn rebuild(self, node: &DomNode, state: &mut Self::State);
}

impl Property for () {
    type State = ();

    fn build(self, _node: &DomNode) -> Self::State {}

    fn rebuild(self, _node: &DomNode, _state: &mut Self::State) {}
}

pub struct Margin(MaybeSignal<u16>);

pub fn margin(margin: impl Into<MaybeSignal<u16>>) -> Margin {
    Margin(margin.into())
}

impl Property for Margin {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let layout_props = node.layout_props();
        RenderEffect::new({
            let margin = self.0;
            let margin = Memo::new(move |_| margin.get());
            move |_| {
                layout_props.borrow_mut().margin = margin.get();
                refresh_dom();
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub struct Flex(MaybeSignal<ratatui::layout::Flex>);

impl Property for Flex {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let layout_props = node.layout_props();
        RenderEffect::new({
            let flex = self.0;
            let flex = Memo::new(move |_| flex.get());
            move |_| {
                layout_props.borrow_mut().flex = flex.get();
                refresh_dom();
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub struct Spacing(MaybeSignal<u16>);

impl Property for Spacing {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let layout_props = node.layout_props();
        RenderEffect::new({
            let spacing = self.0;
            let spacing = Memo::new(move |_| spacing.get());
            move |_| {
                layout_props.borrow_mut().spacing = spacing.get();
                refresh_dom();
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub struct Block(MaybeSignal<ratatui::widgets::Block<'static>>);

pub fn block(block: impl Into<MaybeSignal<ratatui::widgets::Block<'static>>>) -> Block {
    Block(block.into())
}

impl Property for Block {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let layout_props = node.layout_props();
        RenderEffect::new({
            let block = self.0;
            let block = Memo::new(move |_| block.get());
            move |_| {
                layout_props.borrow_mut().block = Some(block.get());
                refresh_dom();
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub struct Focusable(pub(crate) MaybeSignal<bool>);

pub fn focusable(focusable: impl Into<MaybeSignal<bool>>) -> Focusable {
    Focusable(focusable.into())
}

impl Property for Focusable {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let focusable_rc = Rc::new(RefCell::new(false));
        node.set_focusable(focusable_rc.clone());
        RenderEffect::new({
            let focusable = self.0;
            let focusable = Memo::new(move |_| focusable.get());
            let focusable_rc = focusable_rc.clone();
            move |_| {
                *focusable_rc.borrow_mut() = focusable.get();
                refresh_dom();
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

#[derive(Debug)]
pub struct Element<C, P> {
    inner: DomNode,
    children: C,
    properties: P,
}

impl<C, P> Element<C, P>
where
    C: NextTuple,
{
    pub fn child<T>(self, child: T) -> Element<C::Output<T>, P> {
        Element {
            inner: self.inner,
            children: self.children.next_tuple(child),
            properties: self.properties,
        }
    }
}

impl<C, P> Element<C, P>
where
    P: NextTuple,
{
    pub fn margin<S>(self, margin: S) -> Element<C, P::Output<Margin>>
    where
        S: Into<MaybeSignal<u16>>,
    {
        Element {
            inner: self.inner,
            children: self.children,
            properties: self.properties.next_tuple(Margin(margin.into())),
        }
    }

    pub fn flex<S>(self, flex: S) -> Element<C, P::Output<Flex>>
    where
        S: Into<MaybeSignal<ratatui::layout::Flex>>,
    {
        Element {
            inner: self.inner,
            children: self.children,
            properties: self.properties.next_tuple(Flex(flex.into())),
        }
    }

    pub fn spacing<S>(self, spacing: S) -> Element<C, P::Output<Spacing>>
    where
        S: Into<MaybeSignal<u16>>,
    {
        Element {
            inner: self.inner,
            children: self.children,
            properties: self.properties.next_tuple(Spacing(spacing.into())),
        }
    }

    pub fn block<S>(self, block: S) -> Element<C, P::Output<Block>>
    where
        S: Into<MaybeSignal<ratatui::widgets::Block<'static>>>,
    {
        Element {
            inner: self.inner,
            children: self.children,
            properties: self.properties.next_tuple(Block(block.into())),
        }
    }

    pub fn focusable<S>(self, focusable: S) -> Element<C, P::Output<Focusable>>
    where
        S: Into<MaybeSignal<bool>>,
    {
        Element {
            inner: self.inner,
            children: self.children,
            properties: self.properties.next_tuple(Focusable(focusable.into())),
        }
    }
}

impl<C, P> Element<C, P> {
    pub fn id(self, id: impl Into<NodeId>) -> Self {
        self.inner.set_id(id);
        self
    }

    pub fn class(self, class: impl Into<String>) -> Self {
        self.inner.set_class(class);
        self
    }

    pub fn on_focus<F>(self, handler: F) -> Self
    where
        F: FnMut(FocusEvent, EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_focus(handler));
        self
    }

    pub fn on_blur<F>(self, handler: F) -> Self
    where
        F: FnMut(BlurEvent, EventData) + 'static,
    {
        self.inner
            .update_event_handlers(|event_handlers| event_handlers.on_blur(handler));
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

pub struct Constraint(pub(crate) MaybeSignal<ratatui::layout::Constraint>);

pub fn constraint(
    constraint: impl Into<MaybeSignal<ratatui::layout::Constraint>>,
) -> (Constraint,) {
    (Constraint(constraint.into()),)
}

pub fn length(length: impl Into<MaybeSignal<u16>>) -> (Constraint,) {
    let length = length.into();
    constraint(derive_signal!(ratatui::layout::Constraint::Length(
        length.get()
    )))
}

pub fn percentage(percentage: impl Into<MaybeSignal<u16>>) -> (Constraint,) {
    let percentage = percentage.into();
    constraint(derive_signal!(ratatui::layout::Constraint::Percentage(
        percentage.get()
    )))
}

pub fn fill(fill: impl Into<MaybeSignal<u16>>) -> (Constraint,) {
    let fill = fill.into();
    constraint(derive_signal!(ratatui::layout::Constraint::Fill(
        fill.get()
    )))
}

pub fn ratio(from: impl Into<MaybeSignal<u32>>, to: impl Into<MaybeSignal<u32>>) -> (Constraint,) {
    let from = from.into();
    let to = to.into();
    constraint(derive_signal!(ratatui::layout::Constraint::Ratio(
        from.get(),
        to.get()
    )))
}

impl Property for Constraint {
    type State = RenderEffect<()>;

    fn build(self, node: &DomNode) -> Self::State {
        let constraint_rc = Rc::new(RefCell::new(ratatui::layout::Constraint::default()));
        node.set_constraint(constraint_rc.clone());
        RenderEffect::new({
            let constraint = self.0;
            let constraint = Memo::new(move |_| constraint.get());
            let constraint_rc = constraint_rc.clone();
            move |_| {
                *constraint_rc.borrow_mut() = constraint.get();
                refresh_dom();
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

impl<C, P> Constrainable for Element<C, P>
where
    P: NextTuple,
{
    type Output = Element<C, P::Output<Constraint>>;

    fn constraint<S>(self, constraint: S) -> Self::Output
    where
        S: Into<MaybeSignal<ratatui::layout::Constraint>>,
    {
        Element {
            inner: self.inner,
            children: self.children,
            properties: self.properties.next_tuple(Constraint(constraint.into())),
        }
    }
}

#[macro_export]
macro_rules! row {
    () => (
        $crate::row((), ())
    );
    (props($($properties:expr),+ $(,)?)) => (
        $crate::row(($($properties),+), ())
    );
    (props($($properties:expr),+), $($children:expr),+ $(,)?) => (
        $crate::row(($($properties),+), ($($children),+))
    );
    (props($($properties:expr),+), $children:expr) => (
        $crate::row(($($properties),+), ($children,))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::row((), ($($children),+))
    );
}

#[macro_export]
macro_rules! col {
    () => (
        $crate::col((), ())
    );
    (props($($properties:expr),+ $(,)?)) => (
        $crate::col(($($properties),+), ())
    );
    (props($($properties:expr),+), $($children:expr),+ $(,)?) => (
        $crate::col(($($properties),+), ($($children),+))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::col((), ($($children),+))
    );
}

#[macro_export]
macro_rules! overlay {
    () => (
        $crate::overlay((), ())
    );
    (props($($properties:expr),+ $(,)?)) => (
        $crate::overlay(($($properties),+), ())
    );
    (props($($properties:expr),+), $($children:expr),+ $(,)?) => (
        $crate::overlay(($($properties),+), ($($children),+))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::overlay((), ($($children),+))
    );
}

#[macro_export]
macro_rules! clear {
    () => (
        $crate::overlay((), $crate::widget_ref!($crate::__widgets::Clear))
    );
    (props($properties:expr), $($x:expr),+ $(,)?) => (
        $crate::overlay($properties, ($crate::widget_ref!($crate::__widgets::Clear),$($x),+))
    );
    (props($($properties:expr),+), $($x:expr),+ $(,)?) => (
        $crate::overlay(($($properties),+), ($crate::widget_ref!($crate::__widgets::Clear),$($x),+))
    );
    ($($x:expr),+ $(,)?) => (
        $crate::overlay((), ($crate::widget_ref!($crate::__widgets::Clear),$($x),+))
    );
}

#[macro_export]
macro_rules! absolute {
    ($x:expr) => (
        $crate::absolute($x,())
    );
    ($x:expr, $y:expr) => (
        $crate::absolute($x,($y,))
    );
    ($x:expr, $($y:expr),+ $(,)?) => (
        $crate::absolute($x,($($y),+))
    );
}

pub fn row<C, P>(props: P, children: C) -> Element<C, P> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::row()),
        children,
        properties: props,
    }
}

pub fn col<C, P>(props: P, children: C) -> Element<C, P> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::col()),
        children,
        properties: props,
    }
}

pub fn overlay<C, P>(props: P, children: C) -> Element<C, P> {
    Element {
        inner: DomNode::from_fragment(DocumentFragment::overlay()),
        children,
        properties: props,
    }
}

pub struct Absolute(Rc<RefCell<(u16, u16)>>, MaybeSignal<(u16, u16)>);

impl Property for Absolute {
    type State = RenderEffect<()>;
    fn build(self, _node: &DomNode) -> Self::State {
        let pos_rc = self.0;

        RenderEffect::new({
            let pos_rc = pos_rc.clone();
            let pos = self.1;
            let pos = Memo::new(move |_| pos.get());
            move |_| {
                *pos_rc.borrow_mut() = pos.get();
                refresh_dom();
            }
        })
    }

    fn rebuild(self, node: &DomNode, state: &mut Self::State) {
        let new = self.build(node);
        *state = new;
    }
}

pub fn absolute<C>(pos: impl Into<MaybeSignal<(u16, u16)>>, children: C) -> Element<C, Absolute> {
    let pos_rc = Rc::new(RefCell::new((0, 0)));
    let prop = Absolute(pos_rc.clone(), pos.into());

    let inner = DomNode::from_fragment(DocumentFragment::absolute(pos_rc));
    Element {
        inner,
        children,
        properties: prop,
    }
}

pub struct ElementState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    node: <DomNode as Render<RooibosDom>>::State,
    prop_state: <P as Property>::State,
    children: <C as Render<RooibosDom>>::State,
}

impl<C, P> AsDomNode for ElementState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    fn as_dom_node(&self) -> &DomNode {
        self.node.as_dom_node()
    }
}

impl<C, P> Mountable<RooibosDom> for ElementState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
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

impl<C, P> Render<RooibosDom> for Element<C, P>
where
    C: RenderAny + 'static,
    P: Property,
{
    type State = ElementState<C, P>;

    fn build(self) -> Self::State {
        let inner_state = self.inner.build();
        let prop_state = self.properties.build(&inner_state.0);
        let mut children_state = self.children.build();
        children_state.mount(&inner_state.0, None);

        ElementState {
            node: inner_state,
            children: children_state,
            prop_state,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        if self.inner == state.node.0 {
            self.inner.rebuild(&mut state.node);
            self.properties
                .rebuild(&state.node.0, &mut state.prop_state);
            self.children.rebuild(&mut state.children);
        } else {
            state.children.unmount();
            let mut children_state = self.children.build();
            children_state.mount(&state.node.0, None);
            state.children = children_state;
        }
    }
}

macro_rules! impl_property_for_tuples {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> Property for ($($ty,)*)
            where $($ty: Property,)*
        {
            type State = ($($ty::State,)*);

            fn build(self, element: &DomNode) -> Self::State {
                #[allow(non_snake_case)]
                let ($($ty,)*) = self;
                ($($ty.build(element),)*)
            }

            fn rebuild(self, element: &DomNode, state: &mut Self::State) {
                paste::paste! {
                    #[allow(non_snake_case)]
                    let ($($ty,)*) = self;
                    #[allow(non_snake_case)]
                    let ($([<state_ $ty:lower>],)*) = state;
                    $($ty.rebuild(element, [<state_ $ty:lower>]));*
                }
            }
        }
    }
}

impl_property_for_tuples!(A);
impl_property_for_tuples!(A, B);
impl_property_for_tuples!(A, B, C);
impl_property_for_tuples!(A, B, C, D);
impl_property_for_tuples!(A, B, C, D, E);
impl_property_for_tuples!(A, B, C, D, E, F);
impl_property_for_tuples!(A, B, C, D, E, F, G);
impl_property_for_tuples!(A, B, C, D, E, F, G, H);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_property_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
impl_property_for_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
