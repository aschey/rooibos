use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use leptos_reactive::create_render_effect;
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

thread_local! {
   pub static DOM: RefCell<DomNode> = RefCell::new(DomNode::root());
}

static NODE_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, PartialEq, Eq)]
pub enum NodeType {
    Layout { direction: Direction, margin: u16 },
    Overlay,
    Component(RefCell<View>),
    Root,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DomNode {
    id: Option<String>,
    node_type: NodeType,
    constraint: Constraint,
    children: Vec<DomNode>,
}

#[derive(Clone)]
pub struct View {
    id: u32,
    f: Rc<RefCell<dyn FnMut(&mut Frame, Rect)>>,
}

pub struct DynChild<CF, V>
where
    CF: Fn() -> V + 'static,
    V: IntoDomNode,
{
    child_fn: CF,
}

impl<CF, V> DynChild<CF, V>
where
    CF: Fn() -> V + 'static,
    V: IntoDomNode,
{
    pub fn new(child_fn: CF) -> Self {
        Self { child_fn }
    }
}

impl<CF, V> IntoDomNode for DynChild<CF, V>
where
    CF: Fn() -> V + 'static,
    V: IntoDomNode,
{
    fn attach(self, children: &mut Vec<DomNode>) {
        let child_fn = self.child_fn;
        let prev = leptos_reactive::SpecialNonReactiveZone::enter();
        let view = child_fn().attach(children);

        create_render_effect(move |prev_id: Option<u32>| {
            let new_view = child_fn().into_view();
            let new_id = new_view.id;
            if let Some(prev_id) = prev_id {
                // if prev_id != new_id {
                DOM.with(|d| {
                    let res = d.borrow().replace(prev_id, new_view);
                    // dbg!(res.is_none());
                })
                // }
            }

            new_id
        });
    }

    // fn into_view(self) -> View {
    //     let child_fn = self.child_fn;
    //     let prev = leptos_reactive::SpecialNonReactiveZone::enter();
    //     let view = child_fn().into_view();
    //     leptos_reactive::SpecialNonReactiveZone::exit(prev);

    //     create_render_effect(move |prev_id: Option<u32>| {
    //         let new_view = child_fn().into_view();
    //         let new_id = new_view.id;
    //         if let Some(prev_id) = prev_id {
    //             // if prev_id != new_id {
    //             DOM.with(|d| {
    //                 let res = d.borrow().replace(prev_id, new_view);
    //                 // dbg!(res.is_none());
    //             })
    //             // }
    //         }

    //         new_id
    //     });
    // }
}

impl Eq for View {}

impl PartialEq for View {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// impl<F, V> IntoView for F
// where
//     F: Fn() -> V + 'static,
//     V: IntoView,
// {
//     fn into_view(self) -> View {
//         DynChild::new(self).into_view()
//     }
// }

impl View {
    pub fn new(f: impl FnMut(&mut Frame, Rect) + 'static) -> Self {
        Self {
            id: 0,
            f: Rc::new(RefCell::new(f)),
        }
    }

    pub fn render(&self, frame: &mut Frame, rect: Rect) {
        (self.f.borrow_mut())(frame, rect)
    }
}

// pub trait IntoView {
//     fn into_view(self) -> View;
// }

// impl<F: 'static> IntoView for F
// where
//     F: FnMut(&mut Frame, Rect),
// {
//     fn into_view(self) -> View {
//         View::new(self)
//     }
// }

pub struct DomWidget<F>
where
    F: FnMut(&mut Frame, Rect),
{
    inner: F,
}

impl<F> DomWidget<F>
where
    F: FnMut(&mut Frame, Rect),
{
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

// impl<F> IntoView for DomWidget<F>
// where
//     F: FnMut(&mut Frame, Rect) + 'static,
// {
//     fn into_view(self) -> View {
//         View::new(self.inner)
//     }
// }

// impl IntoView for View {
//     fn into_view(self) -> View {
//         self
//     }
// }

// impl IntoView for DomNode {
//     fn into_view(self) -> View {
//         View::new(move |frame, rect| self.render(frame, rect))
//     }
// }

pub trait IntoDomNode {
    fn attach(self, children: &mut Vec<DomNode>);
}

impl IntoDomNode for DomNode {
    fn attach(self, children: &mut Vec<DomNode>) {
        children.push(self);
    }
}

impl IntoDomNode for View {
    fn attach(self, children: &mut Vec<DomNode>) {
        children.push(DomNode::component(self));
    }
}

impl IntoDomNode for Vec<DomNode> {
    fn attach(self, children: &mut Vec<DomNode>) {
        for node in self.into_iter() {
            children.push(node);
        }
    }
}

impl<F: 'static> IntoDomNode for DomWidget<F>
where
    F: FnMut(&mut Frame, Rect),
{
    fn attach(self, children: &mut Vec<DomNode>) {
        self.into_view().attach(children);
    }
}

impl DomNode {
    fn root() -> Self {
        Self {
            id: None,
            node_type: NodeType::Root,
            children: vec![],
            constraint: Constraint::Min(0),
        }
    }

    pub fn component(v: View) -> Self {
        Self {
            id: None,
            node_type: NodeType::Component(RefCell::new(v.into_view())),
            children: vec![],
            constraint: Constraint::Min(0),
        }
    }

    pub fn row() -> Self {
        Self {
            id: None,
            node_type: NodeType::Layout {
                direction: Direction::Horizontal,
                margin: 0,
            },
            children: vec![],
            constraint: Constraint::Min(0),
        }
    }

    pub fn col() -> Self {
        Self {
            id: None,
            node_type: NodeType::Layout {
                direction: Direction::Vertical,
                margin: 0,
            },
            children: vec![],
            constraint: Constraint::Min(0),
        }
    }

    pub fn overlay() -> Self {
        Self {
            id: None,
            node_type: NodeType::Overlay,
            children: vec![],
            constraint: Constraint::Min(0),
        }
    }

    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn render(&self, frame: &mut Frame, rect: Rect) {
        match &self.node_type {
            NodeType::Layout { direction, margin } => {
                let layout = Layout::default().direction(*direction).margin(*margin);
                let constraints = self.children.iter().map(|c| c.constraint);
                let chunks = layout
                    .constraints(constraints.collect::<Vec<_>>())
                    .split(rect);
                self.children
                    .iter()
                    .zip(chunks.iter())
                    .for_each(|(child, chunk)| {
                        child.render(frame, *chunk);
                    });
            }
            NodeType::Overlay | NodeType::Root => {
                self.children.iter().for_each(|child| {
                    child.render(frame, rect);
                });
            }
            NodeType::Component(component) => {
                component.borrow().render(frame, rect);
            }
        }
    }

    pub fn margin(mut self, new_margin: u16) -> Self {
        if let NodeType::Layout { margin, .. } = &mut self.node_type {
            *margin = new_margin;
        }
        self
    }

    pub fn child(mut self, node: impl IntoDomNode) -> Self {
        node.attach(&mut self.children);
        self
    }

    fn matches_id(&self, id: u32) -> bool {
        if let NodeType::Component(component) = &self.node_type {
            return component.borrow().id == id;
        }
        false
    }

    fn replace_view(&self, view: View) {
        if let NodeType::Component(component) = &self.node_type {
            println!("HI");
            *component.borrow_mut() = view;
        }
    }

    pub fn replace(&self, id: u32, mut new: View) -> Option<View> {
        if self.matches_id(id) {
            self.replace_view(new);
            return None;
        }
        for child in self.children.iter() {
            match child.replace(id, new) {
                Some(returned) => {
                    new = returned;
                }
                None => {
                    return None;
                }
            }
        }
        Some(new)
    }
}

pub fn mount(v: impl IntoDomNode) {
    DOM.with(|d| *d = *d.borrow_mut().child(v));
}

pub fn render_dom(frame: &mut Frame) {
    DOM.with(|d| d.borrow().render(frame, frame.size()));
}
