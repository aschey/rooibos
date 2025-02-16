use std::cell::RefCell;

use ratatui::layout::{Position, Rect};
use terminput::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::{
    ClickEvent, ClickEventFn, ClickEventProps, EventData, EventHandle, EventHandlers, KeyEventProps,
};
use crate::{
    DomNodeKey, MatchBehavior, NodeId, NodeProperties, NodeType, focus_next, focus_prev,
    set_pending_resize, toggle_print_dom, trigger_window_focus_changed, with_nodes, with_nodes_mut,
};

thread_local! {
    static EVENT_DISPATCHER: RefCell<EventDispatcher> = RefCell::new(EventDispatcher::new())
}

struct EventDispatcher {
    last_mouse_position: Position,
}

pub fn dispatch_event(event: Event) {
    EVENT_DISPATCHER.with(|e| e.borrow_mut().dispatch(event))
}

pub(crate) fn reset_mouse_position() {
    EVENT_DISPATCHER.with(|e| e.borrow_mut().reset_mouse_position())
}

impl EventDispatcher {
    fn new() -> Self {
        Self {
            last_mouse_position: Position {
                x: u16::MAX,
                y: u16::MAX,
            },
        }
    }

    fn reset_mouse_position(&mut self) {
        self.dispatch(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: self.last_mouse_position.x,
            row: self.last_mouse_position.y,
            modifiers: KeyModifiers::empty(),
        }));
    }

    fn dispatch(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => {
                dispatch_key_event(key_event);
            }
            Event::FocusGained => {
                trigger_window_focus_changed(true);
            }
            Event::FocusLost => {
                trigger_window_focus_changed(false);
            }
            Event::Mouse(mouse_event) => {
                self.dispatch_mouse_event(mouse_event);
            }
            Event::Paste(val) => {
                dispatch_paste(val);
            }
            Event::Resize { .. } => {
                set_pending_resize();
            }
        };
    }

    fn dispatch_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::Down(mouse_button) => {
                dispatch_mouse_down(mouse_event, mouse_button);
            }
            MouseEventKind::Up(_) => {}
            MouseEventKind::Drag(_) => {}
            MouseEventKind::Moved => {
                self.dispatch_mouse_moved(mouse_event);
            }
            MouseEventKind::Scroll(_) => {
                self.dispatch_scroll_event(mouse_event);
            }
        }
    }

    fn dispatch_mouse_moved(&mut self, mouse_event: MouseEvent) {
        self.last_mouse_position.x = mouse_event.column;
        self.last_mouse_position.y = mouse_event.row;
        let roots = with_nodes(|nodes| nodes.roots_desc());
        let mut current = None;

        for root in roots {
            let found = root.get_key().traverse(
                |key, inner| {
                    if !inner.enabled() {
                        return None;
                    }

                    if inner.position().contains(Position {
                        x: mouse_event.column,
                        y: mouse_event.row,
                    }) && inner.focusable()
                    {
                        Some(key)
                    } else {
                        None
                    }
                },
                MatchBehavior::SearchChildrenOnMatch,
            );
            if let Some(found) = found.first() {
                current = Some(*found);
                break;
            }
        }

        let hovered_key = with_nodes(|nodes| nodes.hovered_key());
        if let Some(current) = current {
            if let Some(hovered_key) = hovered_key {
                if hovered_key != current {
                    bubble_event(
                        hovered_key,
                        |props| props.event_handlers.on_mouse_leave.clone(),
                        |event, node_id, rect, handle| {
                            event.borrow_mut()(
                                EventData {
                                    target: node_id,
                                    rect,
                                },
                                handle,
                            );
                        },
                    );
                }
            }
            if hovered_key != Some(current) {
                with_nodes_mut(|nodes| nodes.set_hovered(current));
                bubble_event(
                    current,
                    |props| props.event_handlers.on_mouse_enter.clone(),
                    |event, node_id, rect, handle| {
                        event.borrow_mut()(
                            EventData {
                                target: node_id,
                                rect,
                            },
                            handle,
                        );
                    },
                );
            }
        } else {
            let hovered_key = with_nodes(|nodes| nodes.hovered_key());
            if let Some(hovered_key) = hovered_key {
                bubble_event(
                    hovered_key,
                    |props| props.event_handlers.on_mouse_leave.clone(),
                    |event, node_id, rect, handle| {
                        event.borrow_mut()(
                            EventData {
                                target: node_id,
                                rect,
                            },
                            handle,
                        );
                    },
                );
                with_nodes_mut(|nodes| {
                    nodes.remove_hovered();
                });
            }
        }
    }

    fn dispatch_scroll_event(&self, mouse_event: MouseEvent) {
        if let Some(current_hovered) = hit_test(
            Position {
                x: mouse_event.column,
                y: mouse_event.row,
            },
            |props| props.max_scroll_offset != Position::ORIGIN,
        )
        .last()
        {
            let MouseEventKind::Scroll(direction) = mouse_event.kind else {
                unreachable!()
            };
            with_nodes_mut(|n| {
                n.scroll(*current_hovered, direction);
            });
            bubble_event(
                *current_hovered,
                |props| props.event_handlers.on_scroll.clone(),
                |event, node_id, rect, handle| {
                    (event.borrow_mut())(
                        direction,
                        EventData {
                            rect,
                            target: node_id,
                        },
                        handle,
                    );
                },
            );
        }
    }
}

fn dispatch_key_event(key_event: KeyEvent) {
    if key_event.code == KeyCode::Tab && key_event.kind == KeyEventKind::Press {
        focus_next();
    } else if key_event.code == KeyCode::Tab
        && key_event.modifiers.intersects(KeyModifiers::SHIFT)
        && key_event.kind == KeyEventKind::Press
    {
        focus_prev();
    } else if key_event.code == KeyCode::Char('x')
        && key_event.modifiers.contains(KeyModifiers::CTRL)
    {
        toggle_print_dom();
    } else if let Some(key) = with_nodes(|nodes| nodes.focused_key()) {
        bubble_key_event(key, key_event);
    } else {
        // No focused node, send the event to the root nodes instead
        let roots = with_nodes(|nodes| nodes.roots_desc());
        for root in roots {
            if bubble_key_event(root.get_key(), key_event) {
                return;
            }
        }
    }
}

fn bubble_key_event(key: DomNodeKey, key_event: KeyEvent) -> bool {
    match key_event.kind {
        KeyEventKind::Press | KeyEventKind::Repeat => bubble_event(
            key,
            |props| props.event_handlers.on_key_down.clone(),
            |event, node_id, rect, handle| {
                event.borrow_mut().handle(KeyEventProps {
                    event: key_event,
                    data: EventData {
                        rect,
                        target: node_id,
                    },
                    handle,
                });
            },
        ),
        KeyEventKind::Release => bubble_event(
            key,
            |props| props.event_handlers.on_key_up.clone(),
            |event, node_id, rect, handle| {
                event.borrow_mut().handle(KeyEventProps {
                    event: key_event,
                    data: EventData {
                        rect,
                        target: node_id,
                    },
                    handle,
                });
            },
        ),
    }
}

fn bubble_event<GE, EF, E>(key: DomNodeKey, get_event: GE, event_fn: EF) -> bool
where
    GE: Fn(&NodeProperties) -> Option<E>,
    EF: Fn(&mut E, Option<NodeId>, Rect, EventHandle),
{
    let enabled = with_nodes(|nodes| nodes[key].enabled());
    if !enabled {
        return false;
    }
    let (rect, node_id, event) = with_nodes(|nodes| {
        (
            *nodes[key].rect.borrow(),
            nodes[key].id.clone(),
            get_event(&nodes[key]),
        )
    });
    let mut handled = false;
    if let Some(mut event) = event {
        handled = true;
        let handle = EventHandle::default();
        event_fn(&mut event, node_id, rect, handle.clone());
        if handle.get_stop_propagation() {
            return handled;
        }
    }
    if let Some(parent) = with_nodes(|nodes| nodes[key].parent) {
        let child_handled = bubble_event(parent, get_event, event_fn);
        handled = handled || child_handled;
    }
    handled
}

fn hit_test<F>(position: Position, filter: F) -> Vec<DomNodeKey>
where
    F: Fn(&NodeProperties) -> bool,
{
    let roots = with_nodes(|nodes| nodes.roots_desc());
    for root in roots {
        let found = root.get_key().traverse(
            |key, inner| {
                if !inner.enabled() || !filter(inner) {
                    return None;
                }
                let inner_rect = with_nodes(|n| n.rect(key).visible_bounds());
                if inner_rect.contains(position) {
                    return Some(key);
                }
                None
            },
            MatchBehavior::SearchChildrenOnMatch,
        );
        if !found.is_empty() {
            return found;
        }
    }
    vec![]
}

fn dispatch_mouse_down(mouse_event: MouseEvent, mouse_button: MouseButton) {
    match mouse_button {
        MouseButton::Left | MouseButton::Unknown => {
            dispatch_mouse_button(mouse_event, |handlers| &handlers.on_click)
        }
        MouseButton::Right => {
            dispatch_mouse_button(mouse_event, |handlers| &handlers.on_right_click)
        }
        MouseButton::Middle => {
            dispatch_mouse_button(mouse_event, |handlers| &handlers.on_middle_click)
        }
    }
}

fn dispatch_mouse_button<GE>(mouse_event: MouseEvent, get_event: GE)
where
    GE: Fn(&EventHandlers) -> &Option<ClickEventFn>,
{
    let found = hit_test(
        Position {
            x: mouse_event.column,
            y: mouse_event.row,
        },
        // Only widgets are actually drawn on the screen, layout types or placeholders
        // can't have click events
        |props| matches!(props.node_type, NodeType::Widget(_)),
    );
    if !found.is_empty() {
        let mut focus_set = false;
        let mut stop_propagation = false;
        for key in found.into_iter().rev() {
            let continue_iter = with_nodes_mut(|nodes| {
                if !focus_set && nodes[key].focusable() {
                    focus_set = true;
                    if nodes.focused_key() != Some(key) {
                        nodes.set_focused(Some(key));
                    }
                    if stop_propagation {
                        return false;
                    }
                }
                if !stop_propagation {
                    if let Some(on_click) = get_event(&nodes[key].event_handlers) {
                        let handle = EventHandle::default();
                        let rect = nodes[key].rect.borrow();
                        let node_id = nodes[key].id.clone();
                        on_click.borrow_mut().handle(ClickEventProps {
                            event: ClickEvent {
                                column: mouse_event.column,
                                row: mouse_event.row,
                                modifiers: mouse_event.modifiers,
                            },
                            data: EventData {
                                rect: *rect,
                                target: node_id,
                            },
                            handle: handle.clone(),
                        });
                        if !stop_propagation {
                            stop_propagation = handle.get_stop_propagation();
                        }
                        if focus_set && stop_propagation {
                            return false;
                        }
                    }
                }
                true
            });
            if !continue_iter {
                break;
            }
        }
    }
}

fn dispatch_paste(val: String) {
    if let Some(key) = with_nodes(|nodes| nodes.focused_key()) {
        bubble_event(
            key,
            |props| props.event_handlers.on_paste.clone(),
            |event, node_id, rect, handle| {
                event.borrow_mut()(
                    val.clone(),
                    EventData {
                        rect,
                        target: node_id,
                    },
                    handle,
                );
            },
        );
    }
}
