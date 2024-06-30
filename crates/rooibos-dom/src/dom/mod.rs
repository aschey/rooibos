use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Paragraph, WidgetRef, Wrap};
use reactive_graph::signal::ReadSignal;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::MaybeSignal;
use slotmap::SlotMap;
use tachys::renderer::{CastFrom, Renderer};
use terminput::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use tokio::sync::watch;

use self::dom_state::DomState;
use crate::{derive_signal, widget_ref, EventData, MouseEventFn};

mod any_view;
mod children;
mod document_fragment;
mod dom_node;
mod dom_state;
mod dom_widget;
mod element;
mod focus;
mod into_view;

pub use any_view::*;
pub use children::*;
pub use document_fragment::*;
pub use dom_node::*;
pub use dom_widget::*;
pub use element::*;
pub use focus::*;
pub use into_view::*;

pub trait RenderAny: tachys::view::Render<RooibosDom> {}

impl<T> RenderAny for T where T: tachys::view::Render<RooibosDom> {}

pub trait Render: tachys::view::Render<RooibosDom, State = DomNode> {}

impl<T> Render for T where T: tachys::view::Render<RooibosDom, State = DomNode> {}

// Reference for focus impl https://github.com/reactjs/rfcs/pull/109/files

static NODE_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) fn next_node_id() -> u32 {
    NODE_ID.fetch_add(1, Ordering::SeqCst)
}

pub trait Constrainable: Sized {
    fn constraint<S>(self, constraint: S) -> Self
    where
        S: Into<MaybeSignal<Constraint>>;

    fn length<S>(self, length: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        let length = length.into();
        self.constraint(derive_signal!(Constraint::Length(length.get())))
    }

    fn percentage<S>(self, percentage: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        let percentage = percentage.into();
        self.constraint(derive_signal!(Constraint::Percentage(percentage.get())))
    }

    fn max<S>(self, max: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        let max = max.into();
        self.constraint(derive_signal!(Constraint::Max(max.get())))
    }

    fn min<S>(self, min: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        let min = min.into();
        self.constraint(derive_signal!(Constraint::Min(min.get())))
    }

    fn fill<S>(self, fill: S) -> Self
    where
        S: Into<MaybeSignal<u16>>,
    {
        let fill = fill.into();
        self.constraint(derive_signal!(Constraint::Fill(fill.get())))
    }

    fn ratio<S1, S2>(self, from: S1, to: S2) -> Self
    where
        S1: Into<MaybeSignal<u32>>,
        S2: Into<MaybeSignal<u32>>,
    {
        let from = from.into();
        let to = to.into();
        self.constraint(derive_signal!(Constraint::Ratio(from.get(), to.get())))
    }
}

#[derive(Debug)]
pub struct DomUpdateReceiver(watch::Receiver<()>);

impl DomUpdateReceiver {
    pub async fn changed(&mut self) -> Result<(), watch::error::RecvError> {
        self.0.changed().await
    }
}

thread_local! {
    static DOM_ROOT: RefCell<Option<DomNode>> = const { RefCell::new(None) };
    static DOM_NODES: RefCell<SlotMap<DomNodeKey, DomNodeInner>> =
        RefCell::new(SlotMap::<DomNodeKey, DomNodeInner>::default());
    static DOM_STATE: RefCell<Option<DomState>> = RefCell::new(Some(Default::default()));
    static DOM_UPDATE_TX: RefCell<watch::Sender<()>> = {
        let (tx, _) = watch::channel(());
        RefCell::new(tx)
    };
    static PRINT_DOM: AtomicBool = const { AtomicBool::new(false) };
    static PENDING_RESIZE: AtomicBool = const { AtomicBool::new(true) };
}

pub fn dom_update_receiver() -> DomUpdateReceiver {
    let rx = DOM_UPDATE_TX.with(|d| d.borrow().subscribe());
    DomUpdateReceiver(rx)
}

fn with_state<F, R>(f: F) -> R
where
    F: FnOnce(&DomState) -> R,
{
    DOM_STATE.with(|s| s.borrow().as_ref().map(f).unwrap())
}

fn with_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut DomState) -> R,
{
    DOM_STATE.with(|s| s.borrow_mut().as_mut().map(f).unwrap())
}
fn with_nodes<F, R>(f: F) -> R
where
    F: FnOnce(&SlotMap<DomNodeKey, DomNodeInner>) -> R,
{
    DOM_NODES.with(|n| f(&n.borrow()))
}

fn with_nodes_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut SlotMap<DomNodeKey, DomNodeInner>) -> R,
{
    DOM_NODES.with(|n| f(&mut n.borrow_mut()))
}

fn with_root<F, R>(f: F) -> R
where
    F: FnOnce(&Option<DomNode>) -> R,
{
    DOM_ROOT.with(|n| f(&n.borrow()))
}

#[derive(Debug)]
pub struct RooibosDom;

impl Renderer for RooibosDom {
    type Node = DomNode;

    type Element = DomNode;

    type Text = DomNode;

    type Placeholder = DomNode;

    fn intern(text: &str) -> &str {
        #[cfg(target_arch = "wasm32")]
        return wasm_bindgen::intern(text);
        #[cfg(not(target_arch = "wasm32"))]
        return text;
    }

    fn create_text_node(text: &str) -> Self::Text {
        let text = text.to_owned();
        widget_ref!(Text::from(text.clone())).inner()
    }

    fn create_placeholder() -> Self::Placeholder {
        DomNode::placeholder()
    }

    fn set_text(node: &Self::Text, text: &str) {
        let text = text.to_string();
        node.replace_fragment(DocumentFragment::widget(
            DomWidgetNode::new::<String, _, _>(move || {
                let text = text.clone();
                move |rect, buf| {
                    text.render_ref(rect, buf);
                }
            }),
        ));
    }

    fn set_attribute(_node: &Self::Element, _name: &str, _value: &str) {
        unimplemented!()
    }

    fn remove_attribute(_node: &Self::Element, _name: &str) {
        unimplemented!()
    }

    fn insert_node(parent: &Self::Element, new_child: &Self::Node, marker: Option<&Self::Node>) {
        parent.insert_before(new_child, marker);
        refresh_dom();
    }

    fn remove_node(_parent: &Self::Element, child: &Self::Node) -> Option<Self::Node> {
        unmount_child(child.key(), true);

        Some(child.clone())
    }

    fn clear_children(parent: &Self::Element) {
        clear_children(parent.key())
    }

    fn remove(node: &Self::Node) {
        unmount_child(node.key(), true);
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        node.get_parent()
    }

    fn first_child(_node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn next_sibling(_node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn log_node(_node: &Self::Node) {
        todo!()
    }
}

impl CastFrom<DomNode> for DomNode {
    fn cast_from(source: DomNode) -> Option<Self> {
        Some(source)
    }
}

impl AsRef<DomNode> for DomNode {
    fn as_ref(&self) -> &DomNode {
        self
    }
}

fn cleanup_removed_nodes(node: &DomNodeKey, remove: bool) {
    with_state_mut(|s| {
        s.cleanup_before_remove(node);
    });
    dom_state::remove_hovered();
    let children = with_nodes(|nodes| nodes[*node].children.clone());

    for child in children {
        cleanup_removed_nodes(&child, remove);
    }
    if remove {
        let removed = with_nodes_mut(|nodes| nodes.remove(*node));
        // We need to make sure we drop the removed node after we release the borrow
        // because the drop impl needs to borrow the node list as well
        drop(removed);
    }
}

fn clear_children(parent: DomNodeKey) {
    let children = with_nodes(|nodes| nodes[parent].children.clone());
    for child in children {
        unmount_child(child, true);
    }
}

fn unmount_child(child: DomNodeKey, cleanup: bool) {
    with_nodes_mut(|nodes| {
        let child_node = &nodes[child];
        if let Some(parent) = child_node.parent {
            let child_pos = nodes[parent]
                .children
                .iter()
                .position(|c| c == &child)
                .unwrap();
            nodes[parent].children.remove(child_pos);
        }
        nodes[child].parent = None;
    });

    cleanup_removed_nodes(&child, cleanup);
    refresh_dom();
}

pub fn print_dom() -> Paragraph<'static> {
    let lines = with_root(|dom| {
        with_nodes(|nodes| print_dom_inner(nodes, dom.as_ref().unwrap().key(), ""))
    });
    Paragraph::new(lines.clone()).wrap(Wrap { trim: false })
}

pub fn root() -> DomNodeRepr {
    with_root(|r| {
        with_nodes(|nodes| {
            let root = r.as_ref().unwrap();
            let node = &nodes[root.key()];
            DomNodeRepr::from_node(root.key(), node)
        })
    })
}

fn print_dom_inner(
    dom_ref: &SlotMap<DomNodeKey, DomNodeInner>,
    key: DomNodeKey,
    indent: &str,
) -> Vec<Line<'static>> {
    let node = &dom_ref[key];
    let NodeTypeStructure { name, attrs } = node.node_type.structure();
    let node_name = node.name.clone();
    if node_name == "Placeholder" {
        return vec![];
    }

    let mut line = format!(
        "{indent}<{node_name} type={name} key={key:?} parent={:?}",
        node.parent
    );

    if let Some(attrs) = attrs {
        line += &format!(" {attrs}");
    }
    line += &format!(" constraint={}>", node.constraint.borrow().clone());

    let mut lines = vec![Line::from(line)];

    let child_indent = format!("{indent}  ");

    for key in &node.children {
        lines.append(&mut print_dom_inner(dom_ref, *key, &child_indent));
    }

    lines.push(Line::from(format!("{indent}</{node_name}>")));

    lines
}

pub fn refresh_dom() {
    DOM_UPDATE_TX.with(|tx| tx.borrow().send(()).ok());
}

pub fn mount<F, M>(f: F)
where
    F: FnOnce() -> M + 'static,
    M: Render<State = DomNode>,
{
    let node = f().build();
    DOM_ROOT.with(|d| *d.borrow_mut() = Some(node));
}

pub fn unmount() {
    DOM_ROOT.with(|d| *d.borrow_mut() = None);
    DOM_STATE.with(|d| *d.borrow_mut() = None);
    with_nodes_mut(|d| (*d).clear());
}

pub fn render_dom(buf: &mut Buffer) {
    if PENDING_RESIZE.with(|p| p.swap(false, Ordering::Relaxed)) {
        with_state(|s| s.set_window_size(buf.area));
    }

    if PRINT_DOM.with(|p| p.load(Ordering::Relaxed)) {
        print_dom().render_ref(buf.area, buf);
    } else {
        with_root(|d| d.as_ref().unwrap().render(buf, buf.area));
    }
}

pub fn focus(id: impl Into<NodeId>) {
    let id = id.into();
    let node = with_nodes(|d| {
        d.iter().find_map(|(k, v)| {
            if v.id.as_ref() == Some(&id) {
                Some(k)
            } else {
                None
            }
        })
    });
    if let Some(node) = node {
        dom_state::set_focused(node);
    }
}

pub fn use_focused_node() -> ReadSignal<Option<NodeId>> {
    with_state(|s| s.focused())
}

pub fn use_window_size() -> ReadSignal<Rect> {
    with_state(|s| s.window_size())
}

struct ClickEvent {
    on_click: Option<MouseEventFn>,
    rect: Rect,
    key: DomNodeKey,
}

pub fn send_event(event: Event) {
    match event {
        Event::Key(key_event) => {
            if key_event.code == KeyCode::Tab && key_event.kind == KeyEventKind::Press {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    focus_prev();
                } else {
                    focus_next();
                }
            } else if key_event.code == KeyCode::Char('p')
                && key_event.modifiers.contains(KeyModifiers::CONTROL)
            {
                PRINT_DOM.with(|p| p.swap(!p.load(Ordering::Relaxed), Ordering::Relaxed));
                refresh_dom();
            } else if let Some(key) = with_state(|s| s.focused_key()) {
                match key_event.kind {
                    KeyEventKind::Press | KeyEventKind::Repeat => {
                        let (rect, mut on_key_down) = with_nodes(|nodes| {
                            (
                                *nodes[key].rect.borrow(),
                                nodes[key].event_handlers.on_key_down.clone(),
                            )
                        });
                        if let Some(on_key_down) = &mut on_key_down {
                            #[cfg(debug_assertions)]
                            let _guard =
                                reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                            on_key_down.borrow_mut()(key_event, EventData { rect });
                        }
                    }
                    KeyEventKind::Release => {
                        let (rect, mut on_key_up) = with_nodes(|nodes| {
                            (
                                *nodes[key].rect.borrow(),
                                nodes[key].event_handlers.on_key_up.clone(),
                            )
                        });
                        if let Some(on_key_up) = &mut on_key_up {
                            #[cfg(debug_assertions)]
                            let _guard =
                                reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                            on_key_up.borrow_mut()(key_event, EventData { rect });
                        }
                    }
                }
            }
        }
        Event::FocusGained => {}
        Event::FocusLost => {}
        Event::Mouse(mouse_event) => match mouse_event.kind {
            MouseEventKind::Down(_) => {
                let current = with_nodes(|nodes| {
                    let mut current: Option<ClickEvent> = None;
                    for key in nodes.keys() {
                        let rect = nodes[key].rect.borrow();
                        if rect.contains(Position {
                            x: mouse_event.column,
                            y: mouse_event.row,
                        }) {
                            let node = &nodes[key];
                            if *node.focusable.borrow() || node.event_handlers.on_click.is_some() {
                                current = Some(ClickEvent {
                                    on_click: node.event_handlers.on_click.clone(),
                                    rect: *rect,
                                    key,
                                });
                            }
                            if node.event_handlers.on_click.is_some() {
                                break;
                            }
                        }
                    }
                    current
                });

                if let Some(ClickEvent {
                    on_click,
                    rect,
                    key,
                }) = current
                {
                    let set_focus = with_nodes(|nodes| {
                        with_state(|state| {
                            state.focused_key() != Some(key) && *nodes[key].focusable.borrow()
                        })
                    });
                    if set_focus {
                        dom_state::set_focused(key);
                    }
                    if let Some(on_click) = on_click {
                        #[cfg(debug_assertions)]
                        let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                        on_click.borrow_mut()(mouse_event, EventData { rect });
                    }
                }
            }
            MouseEventKind::Up(_) => {}
            MouseEventKind::Drag(_) => {}
            MouseEventKind::Moved => {
                let current = with_nodes(|nodes| {
                    nodes.keys().find(|k| {
                        nodes[*k].rect.borrow().contains(Position {
                            x: mouse_event.column,
                            y: mouse_event.row,
                        }) && *nodes[*k].focusable.borrow()
                    })
                });

                if let Some(current) = current {
                    let set_focus = with_state(|state| state.hovered_key() != Some(current));
                    if set_focus {
                        dom_state::set_hovered(current);
                    }
                } else {
                    dom_state::remove_hovered();
                }
            }
            MouseEventKind::ScrollDown => {}
            MouseEventKind::ScrollUp => {}
            MouseEventKind::ScrollLeft => {}
            MouseEventKind::ScrollRight => {}
        },
        Event::Paste(_) => {}
        Event::Resize(_, _) => {
            PENDING_RESIZE.with(|p| p.store(true, Ordering::Relaxed));
            refresh_dom();
        }
    };
}

pub fn focus_id(id: impl Into<NodeId>) {
    let id = id.into();
    let found_node = with_nodes(|nodes| {
        nodes.iter().find_map(|(key, node)| {
            if let Some(current_id) = &node.id {
                if &id == current_id {
                    return Some(key);
                }
            }
            None
        })
    });
    if let Some(found_node) = found_node {
        dom_state::set_focused(found_node);
    }
}

pub fn focus_next() {
    let focusable_nodes = with_state(|state| state.focusable_nodes().clone());
    if let Some(focused) = with_state(|state| state.focused_key()) {
        let current_focused = focusable_nodes.iter().position(|n| n == &focused).unwrap();

        if current_focused < focusable_nodes.len() - 1 {
            let next = focusable_nodes[current_focused + 1];
            dom_state::set_focused(next);
            return;
        }
    }
    if let Some(next) = focusable_nodes.first() {
        dom_state::set_focused(*next);
    }
}

pub fn focus_prev() {
    let focusable_nodes = with_state(|state| state.focusable_nodes().clone());
    if let Some(focused) = with_state(|state| state.focused_key()) {
        let current_focused = focusable_nodes.iter().position(|n| n == &focused).unwrap();
        if current_focused > 0 {
            let prev = focusable_nodes[current_focused - 1];
            dom_state::set_focused(prev);
            return;
        }
    }
    if let Some(prev) = focusable_nodes.last() {
        dom_state::set_focused(*prev);
    }
}
