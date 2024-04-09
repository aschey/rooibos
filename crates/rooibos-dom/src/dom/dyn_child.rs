// use std::cell::{OnceCell, RefCell};
// use std::rc::Rc;

// use leptos_reactive::create_render_effect;

// use super::document_fragment::DocumentFragment;
// use super::dom_node::DomNode;
// use crate::dom::dom_node::DomNodeKey;
// use crate::dom::{mount_child, unmount_child};
// use crate::{next_node_id, IntoView, MountKind, Mountable, View};

// pub struct DynChild<CF, V>
// where
//     CF: Fn() -> V + 'static,
//     V: IntoView,
// {
//     id: u32,
//     child_fn: CF,
//     name: String,
// }

// impl<CF, N> DynChild<CF, N>
// where
//     CF: Fn() -> N + 'static,
//     N: IntoView,
// {
//     pub fn new(name: impl Into<String>, child_fn: CF) -> Self {
//         Self {
//             child_fn,
//             id: next_node_id(),
//             name: name.into(),
//         }
//     }
// }

// impl<CF, N> IntoView for DynChild<CF, N>
// where
//     CF: Fn() -> N + 'static,
//     N: IntoView,
// {
//     fn into_view(self) -> View {
//         fn create_dyn_view(
//             component: DynChildRepr,
//             child_fn: Box<dyn Fn() -> View>,
//         ) -> DynChildRepr {
//             let fragment = component.document_fragment.clone();
//             let child = component.child.clone();

//             create_render_effect(move |prev_key: Option<DomNodeKey>| {
//                 let new_child = child_fn().into_view();
//                 let mut child_borrow = (*child).borrow_mut();

//                 if let Some(prev_key) = prev_key {
//                     let prev_child = child_borrow.take().unwrap();

//                     if prev_child != new_child {
//                         unmount_child(prev_key);

//                         let new_key = mount_child(MountKind::Append(&fragment), &new_child);

//                         **child_borrow = Some(new_child);
//                         new_key
//                     } else {
//                         prev_key
//                     }
//                 } else {
//                     let new = mount_child(MountKind::Append(&fragment), &new_child);
//                     **child_borrow = Some(new_child);
//                     new
//                 }
//             });
//             component
//         }

//         let Self { id, child_fn, name } = self;

//         let component = DynChildRepr::new_with_id(id, name);
//         let component = create_dyn_view(component, Box::new(move || child_fn().into_view()));

//         View::DynChild(component)
//     }
// }

// #[derive(Clone, PartialEq, Eq)]
// pub struct DynChildRepr {
//     document_fragment: DomNode,
//     pub(crate) child: Rc<RefCell<Box<Option<View>>>>,
//     pub(crate) id: u32,
// }

// impl DynChildRepr {
//     fn new_with_id(id: u32, name: impl Into<String>) -> Self {
//         let document_fragment = DocumentFragment::transparent(name);

//         Self {
//             document_fragment: DomNode::from_fragment(document_fragment),
//             child: Default::default(),
//             id,
//         }
//     }

//     pub(crate) fn set_name(&mut self, name: impl Into<String>) {
//         let name = name.into();
//     }
// }

// impl Mountable for DynChildRepr {
//     fn get_mountable_node(&self) -> DomNode {
//         self.document_fragment.clone()
//     }
// }

// impl<F, N> IntoView for F
// where
//     F: Fn() -> N + 'static,
//     N: IntoView,
// {
//     fn into_view(self) -> View {
//         DynChild::new("Fn", self).into_view()
//     }
// }
