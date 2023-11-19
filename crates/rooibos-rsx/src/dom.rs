use std::cell::RefCell;
use std::rc::Rc;

use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

enum NodeType {
    Layout(Direction),
    Component(Box<dyn View>),
}

struct DomNode {
    id: String,
    node_type: NodeType,
    constraint: Constraint,
    children: Vec<DomNode>,
}

pub trait View: 'static {
    fn view(&mut self, frame: &mut Frame, rect: Rect);
}

trait IntoView {
    fn class(&self) -> String;
    fn into_view(self) -> Box<dyn View>;
}

impl DomNode {
    fn root<F, V>(id: String, f: F) -> Self
    where
        F: Fn() -> V + 'static,
        V: IntoView,
    {
        Self {
            id,
            node_type: NodeType::Component(f().into_view()),
            children: vec![],
            constraint: Constraint::Min(0),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.node_type {
            NodeType::Layout(direction) => {
                let layout = Layout::default().direction(*direction);
                let constraints = self.children.iter().map(|c| c.constraint);
                let chunks = layout
                    .constraints(constraints.collect::<Vec<_>>())
                    .split(rect);
                self.children
                    .iter_mut()
                    .zip(chunks.iter())
                    .for_each(|(child, chunk)| {
                        child.render(f, *chunk);
                    });
            }
            NodeType::Component(component) => {
                component.view(f, rect);
            }
        }
    }

    fn child(mut self, node: DomNode) -> Self {
        self.children.push(node);
        self
    }
}

struct DomTree {
    root: DomNode,
}
