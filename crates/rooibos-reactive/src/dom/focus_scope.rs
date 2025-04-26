//  col![
//      style(max_width(50), height(full()), overflow_y(scroll())),
//      row![
//          Button::new()
//              .on_click(add_counter)
//              .render(text!("Add Counter")),
//          ],
//          focus_scope![
//              style(focus_list()),
//              for_each(
//                  move || ids.get(),
//                  |k| *k,
//                  move |i| counter(NodeId::new(i.to_string()), move || remove_id(i))
//              )
//          ]
//     ]
//
// disable focus change on tab
//
//  row![
//      col![..],
//      col![..],
//      col![style(focus_none())]
//  ]
//
// modals
//
// focus_scope![
//   style(contain()),
//   col![..]
// ]
//
// grid
//
// grid [
//    focus_scope![
//       style(focus_list()),
//       // arrow up/down to focus
//       col![
//         rows.map(|r| grid_row(r))
//       ]
//    ]
// ]
//
// fn grid_row() {
//   focus_scope![
//     style(focus_list()),
//     // arrow left/right to focus
//     row![
//       item1,
//       item2
//     ]
//   ]
// }

use next_tuple::NextTuple;
use rooibos_dom::AsDomNode;
use tachys::prelude::Renderer;
use tachys::view::{Mountable, Render};

use super::layout::Property;
use super::{DomNode, RenderAny, RooibosDom};

pub struct FocusScope<C, P> {
    inner: DomNode,
    children: C,
    properties: P,
}

impl<C, P> FocusScope<C, P>
where
    C: NextTuple,
{
    pub fn child<T>(self, child: T) -> FocusScope<C::Output<T>, P> {
        FocusScope {
            inner: self.inner,
            children: self.children.next_tuple(child),
            properties: self.properties,
        }
    }
}

pub fn focus_scope<C, P>(props: P, children: C) -> FocusScope<C, P> {
    FocusScope {
        inner: DomNode(rooibos_dom::DomNode::focus_scope()),
        children,
        properties: props,
    }
}

#[macro_export]
macro_rules! focus_scope {
    () => (
        $crate::dom::focus_scope::focus_scope((), ())
    );
    (style($($properties:expr),+ $(,)?)) => (
        $crate::dom::focus_scope::focus_scope(($($properties),+), ())
    );
    (style($($properties:expr),+ $(,)?), $($children:expr),+ $(,)?) => (
        $crate::dom::focus_scope::focus_scope(($($properties),+), ($($children),+))
    );
    (style($($properties:expr),+ $(,)?), $children:expr) => (
        $crate::dom::focus_scope::focus_scope(($($properties),+), ($children,))
    );
    ($($children:expr),+ $(,)?) => (
        $crate::dom::focus_scope::focus_scope((), ($($children),+))
    );
}

pub struct FocusScopeState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    node: <DomNode as Render<RooibosDom>>::State,
    prop_state: <P as Property>::State,
    children: <C as Render<RooibosDom>>::State,
}

impl<C, P> AsDomNode for FocusScopeState<C, P>
where
    C: Render<RooibosDom>,
    P: Property,
{
    fn as_dom_node(&self) -> &rooibos_dom::DomNode {
        self.node.as_dom_node()
    }
}

impl<C, P> Mountable<RooibosDom> for FocusScopeState<C, P>
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

impl<C, P> Render<RooibosDom> for FocusScope<C, P>
where
    C: RenderAny,
    P: Property + 'static,
{
    type State = FocusScopeState<C, P>;

    fn build(self) -> Self::State {
        let inner_state = self.inner.build();
        let prop_state = self.properties.build(&inner_state.0);
        let mut children_state = self.children.build();
        children_state.mount(&inner_state.0, None);

        FocusScopeState {
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
