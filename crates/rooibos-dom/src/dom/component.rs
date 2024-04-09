// // use std::cell::OnceCell;
// // use std::rc::Rc;

// // use super::document_fragment::DocumentFragment;
// // use super::dom_node::DomNode;
// // use super::mount_child;
// // use crate::{next_node_id, IntoView, MountKind, Mountable, View};

// use tachys::prelude::*;

// use super::{mount_child, unmount_child};
// use crate::{Element, RooibosDom, ToDomNode};

// pub struct Component<Props, Child> {
//     pub(crate) properties: Props,
//     pub(crate) children: Child,
//     pub(crate) element: Element,
// }

// pub struct ComponentState<Props, Child>
// where
//     Child: Render<RooibosDom>,
//     Props: Render<RooibosDom>,
// {
//     element: Element,
//     properties: Props::State,
//     children: Child::State,
// }

// impl<Props, Child> Mountable<RooibosDom> for ComponentState<Props, Child>
// where
//     Child: Render<RooibosDom>,
//     Props: Render<RooibosDom>,
// {
//     fn unmount(&mut self) {
//         self.children.unmount();
//         unmount_child(self.element.to_dom_node().key());
//     }

//     fn mount(
//         &mut self,
//         parent: &<RooibosDom as Renderer>::Element,
//         marker: Option<&<RooibosDom as Renderer>::Node>,
//     ) {
//         let dom_node = self.element.to_dom_node();
//         self.children.mount(&dom_node, marker);
//         mount_child(crate::MountKind::Append(parent), &dom_node);
//     }

//     fn insert_before_this(
//         &self,
//         parent: &<RooibosDom as Renderer>::Element,
//         child: &mut dyn Mountable<RooibosDom>,
//     ) -> bool {
//         child.mount(parent, Some(&self.element.to_dom_node()));
//         true
//     }
// }

// impl<Props, Child> Render<RooibosDom> for Component<Props, Child>
// where
//     Child: Render<RooibosDom>,
//     Props: Render<RooibosDom>,
// {
//     type State = ComponentState<Props, Child>;

//     type FallibleState = ComponentState<Props, Child>;

//     type AsyncOutput = ();

//     fn build(self) -> Self::State {
//         ComponentState {
//             properties: self.properties.build(),
//             element: self.element,
//             children: self.children.build(),
//         }
//     }

//     fn rebuild(self, state: &mut Self::State) {
//         self.properties.rebuild(&mut state.properties);
//         self.children.rebuild(&mut state.children);
//     }

//     fn try_build(self) -> any_error::Result<Self::FallibleState> {
//         Ok(self.build())
//     }

//     fn try_rebuild(self, state: &mut Self::FallibleState) -> any_error::Result<()> {
//         self.rebuild(state);
//         Ok(())
//     }

//     async fn resolve(self) -> Self::AsyncOutput {}
// }

// // #[derive(Clone, PartialEq, Eq)]
// // pub struct ComponentRepr {
// //     document_fragment: DomNode,
// //     mounted: Rc<OnceCell<DomNode>>,
// //     children: Vec<View>,
// //     id: u32,
// // }

// // impl ComponentRepr {
// //     pub fn new_with_id(name: impl Into<String>, id: u32, children: Vec<View>) -> Self {
// //         let name = name.into();
// //         let document_fragment = DocumentFragment::transparent(name.clone());

// //         Self {
// //             document_fragment: DomNode::from_fragment(document_fragment),
// //             mounted: Default::default(),
// //             children,
// //             id,
// //         }
// //     }

// //     pub(crate) fn set_name(&mut self, name: impl Into<String>) {
// //         let name = name.into();
// //     }
// // }

// // impl Mountable for ComponentRepr {
// //     fn get_mountable_node(&self) -> DomNode {
// //         if let Some(mounted) = self.mounted.get() {
// //             mounted.clone()
// //         } else {
// //             for child in &self.children {
// //                 mount_child(MountKind::Append(&self.document_fragment), child);
// //             }
// //             let node = self.document_fragment.clone();
// //             self.mounted.set(node.clone()).unwrap();
// //             node
// //         }
// //     }
// // }

// // impl IntoView for ComponentRepr {
// //     fn into_view(self) -> View {
// //         View::Component(self)
// //     }
// // }

// // pub struct Component<F, V>
// // where
// //     F: FnOnce() -> V,
// //     V: IntoView,
// // {
// //     id: u32,
// //     name: String,
// //     children_fn: F,
// // }

// // impl<F, V> Component<F, V>
// // where
// //     F: FnOnce() -> V,
// //     V: IntoView,
// // {
// //     /// Creates a new component.
// //     pub fn new(name: impl Into<String>, f: F) -> Self {
// //         Self {
// //             id: next_node_id(),
// //             name: name.into(),
// //             children_fn: f,
// //         }
// //     }
// // }

// // impl<F, V> IntoView for Component<F, V>
// // where
// //     F: FnOnce() -> V,
// //     V: IntoView,
// // {
// //     fn into_view(self) -> View {
// //         let Self {
// //             id,
// //             name,
// //             children_fn,
// //         } = self;

// //         // disposed automatically when the parent scope is disposed
// //         let child = untrack_with_diagnostics(|| children_fn().into_view());
// //         let repr = ComponentRepr::new_with_id(name, id, vec![child]);

// //         repr.into_view()
// //     }
// // }
