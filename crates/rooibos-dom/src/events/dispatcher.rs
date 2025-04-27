use std::cell::RefCell;

use ratatui::layout::{Position, Rect};
use terminput::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    ScrollDirection,
};

use super::{
    ClickEvent, ClickEventFn, ClickEventProps, Event, EventData, EventHandle, EventHandlers,
    KeyEventProps,
};
use crate::{
    DomNodeKey, FocusEventType, MatchBehavior, NodeId, NodeProperties, NodeType, focus_next,
    focus_next_list, focus_prev, focus_prev_list, push_pending_event, refresh_dom,
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

pub fn queue_event(event: Event) {
    push_pending_event(event);
    refresh_dom();
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
            Event::WindowFocusGained => {
                trigger_window_focus_changed(true);
            }
            Event::WindowFocusLost => {
                trigger_window_focus_changed(false);
            }
            Event::Mouse(mouse_event) => {
                self.dispatch_mouse_event(mouse_event);
            }
            Event::Paste(val) => {
                dispatch_paste(val);
            }
            Event::Resize => {
                set_pending_resize();
            }
            Event::NodeEnable(key) => {
                dispatch_node_enable(key);
            }
            Event::NodeDisable(key) => {
                dispatch_node_disable(key);
            }
        };
    }

    fn dispatch_mouse_event(&mut self, mouse_event: MouseEvent) {
        let position = Position {
            x: mouse_event.column,
            y: mouse_event.row,
        };
        match mouse_event.kind {
            MouseEventKind::Down(mouse_button) => {
                dispatch_mouse_down(position, mouse_event.modifiers, mouse_button);
            }
            MouseEventKind::Up(_) => {}
            MouseEventKind::Drag(_) => {}
            MouseEventKind::Moved => {
                self.dispatch_mouse_moved(position);
            }
            MouseEventKind::Scroll(direction) => {
                self.dispatch_scroll_event(position, direction);
            }
        }
    }

    fn dispatch_mouse_moved(&mut self, position: Position) {
        self.last_mouse_position = position;
        let roots = with_nodes(|nodes| nodes.roots_desc());
        let mut current = None;

        for root in roots {
            let found = root.get_key().traverse(
                |key, inner| {
                    if matches!(inner.node_type, NodeType::Widget(_))
                        && inner.visible()
                        && inner.position().contains(position)
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
                            for handler in event {
                                handler.borrow_mut()(
                                    EventData {
                                        target: node_id.clone(),
                                        rect,
                                    },
                                    handle.clone(),
                                );
                            }
                        },
                        // Still run mouse move events on disabled nodes because we may need to
                        // remove some hover state and if the node loses hover while disabled,
                        // these events would never fire
                        AllowDisabled::Allow,
                    );
                }
            }
            if hovered_key != Some(current) {
                with_nodes_mut(|nodes| nodes.set_hovered(current));
                bubble_event(
                    current,
                    |props| props.event_handlers.on_mouse_enter.clone(),
                    |event, node_id, rect, handle| {
                        for handler in event {
                            handler.borrow_mut()(
                                EventData {
                                    target: node_id.clone(),
                                    rect,
                                },
                                handle.clone(),
                            );
                        }
                    },
                    AllowDisabled::Allow,
                );
            }
        } else {
            let hovered_key = with_nodes(|nodes| nodes.hovered_key());
            if let Some(hovered_key) = hovered_key {
                bubble_event(
                    hovered_key,
                    |props| props.event_handlers.on_mouse_leave.clone(),
                    |event, node_id, rect, handle| {
                        for handler in event {
                            handler.borrow_mut()(
                                EventData {
                                    target: node_id.clone(),
                                    rect,
                                },
                                handle.clone(),
                            );
                        }
                    },
                    AllowDisabled::Allow,
                );
                with_nodes_mut(|nodes| {
                    nodes.remove_hovered();
                });
            }
        }
    }

    fn dispatch_scroll_event(&self, position: Position, direction: ScrollDirection) {
        if let Some(current_hovered) = hit_test(position, |props| {
            props.max_scroll_offset != Position::ORIGIN
        })
        .last()
        {
            with_nodes_mut(|n| {
                n.scroll(*current_hovered, direction);
            });
            bubble_event(
                *current_hovered,
                |props| props.event_handlers.on_scroll.clone(),
                |event, node_id, rect, handle| {
                    for handler in event {
                        (handler.borrow_mut())(
                            direction,
                            EventData {
                                rect,
                                target: node_id.clone(),
                            },
                            handle.clone(),
                        );
                    }
                },
                AllowDisabled::Disallow,
            );
        }
    }
}

fn dispatch_key_event(key_event: KeyEvent) {
    match with_nodes(|n| n.focus_event_type(&key_event)) {
        Some(FocusEventType::Next) => {
            focus_next();
            return;
        }
        Some(FocusEventType::Previous) => {
            focus_prev();
            return;
        }
        Some(FocusEventType::NextList) => {
            focus_next_list();
            return;
        }
        Some(FocusEventType::PreviousList) => {
            focus_prev_list();
            return;
        }
        None => {}
    }

    if key_event.code == KeyCode::Char('x') && key_event.modifiers.contains(KeyModifiers::CTRL) {
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

fn dispatch_node_enable(key: DomNodeKey) {
    bubble_event(
        key,
        |props| props.event_handlers.on_enable.clone(),
        |event, node_id, rect, handle| {
            for on_enable in event {
                on_enable.borrow_mut()(
                    EventData {
                        rect,
                        target: node_id.clone(),
                    },
                    handle.clone(),
                );
            }
        },
        AllowDisabled::Disallow,
    );
}

fn dispatch_node_disable(key: DomNodeKey) {
    bubble_event(
        key,
        |props| props.event_handlers.on_disable.clone(),
        |event, node_id, rect, handle| {
            for on_enable in event {
                on_enable.borrow_mut()(
                    EventData {
                        rect,
                        target: node_id.clone(),
                    },
                    handle.clone(),
                );
            }
        },
        AllowDisabled::Allow,
    );
}

fn bubble_key_event(key: DomNodeKey, key_event: KeyEvent) -> bool {
    match key_event.kind {
        KeyEventKind::Press | KeyEventKind::Repeat => bubble_event(
            key,
            |props| props.event_handlers.on_key_down.clone(),
            |event, node_id, rect, handle| {
                for handler in event {
                    handler.borrow_mut().handle(KeyEventProps {
                        event: key_event,
                        data: EventData {
                            rect,
                            target: node_id.clone(),
                        },
                        handle: handle.clone(),
                    });
                }
            },
            AllowDisabled::Disallow,
        ),
        KeyEventKind::Release => bubble_event(
            key,
            |props| props.event_handlers.on_key_up.clone(),
            |event, node_id, rect, handle| {
                for handler in event {
                    handler.borrow_mut().handle(KeyEventProps {
                        event: key_event,
                        data: EventData {
                            rect,
                            target: node_id.clone(),
                        },
                        handle: handle.clone(),
                    });
                }
            },
            AllowDisabled::Disallow,
        ),
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum AllowDisabled {
    Allow,
    Disallow,
}

pub(crate) fn bubble_event<GE, EF, E>(
    key: DomNodeKey,
    get_event: GE,
    event_fn: EF,
    allow_disabled: AllowDisabled,
) -> bool
where
    GE: Fn(&NodeProperties) -> Vec<E>,
    EF: Fn(&mut Vec<E>, Option<NodeId>, Rect, EventHandle),
{
    if allow_disabled == AllowDisabled::Disallow {
        let enabled = with_nodes(|nodes| nodes[key].enabled());
        if !enabled {
            return false;
        }
    }
    let (rect, node_id, mut event) = with_nodes(|nodes| {
        (
            *nodes[key].rect.borrow(),
            nodes[key].id.clone(),
            get_event(&nodes[key]),
        )
    });
    let mut handled = false;
    if !event.is_empty() {
        handled = true;
        let handle = EventHandle::default();
        event_fn(&mut event, node_id, rect, handle.clone());
        if handle.get_stop_propagation() {
            return handled;
        }
    }
    if let Some(parent) = with_nodes(|nodes| nodes[key].parent) {
        let child_handled = bubble_event(parent, get_event, event_fn, allow_disabled);
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
                let contains_position = with_nodes(|n| {
                    n.try_rect(key)
                        .map(|r| r.visible_bounds().contains(position))
                })
                .unwrap_or(false);
                if contains_position {
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

fn dispatch_mouse_down(position: Position, modifiers: KeyModifiers, mouse_button: MouseButton) {
    match mouse_button {
        MouseButton::Left | MouseButton::Unknown => {
            dispatch_mouse_button(position, modifiers, |handlers| &handlers.on_click)
        }
        MouseButton::Right => {
            dispatch_mouse_button(position, modifiers, |handlers| &handlers.on_right_click)
        }
        MouseButton::Middle => {
            dispatch_mouse_button(position, modifiers, |handlers| &handlers.on_middle_click)
        }
    }
}

fn dispatch_mouse_button<GE>(position: Position, modifiers: KeyModifiers, get_event: GE)
where
    GE: Fn(&EventHandlers) -> &Vec<ClickEventFn>,
{
    let found = hit_test(position, |props| {
        props.focusable() || !get_event(&props.event_handlers).is_empty()
    });
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
                    let on_click = get_event(&nodes[key].event_handlers);
                    let handle = EventHandle::default();
                    let rect = nodes[key].rect.borrow();
                    let node_id = nodes[key].id.clone();
                    for handler in on_click {
                        handler.borrow_mut().handle(ClickEventProps {
                            event: ClickEvent {
                                column: position.x,
                                row: position.y,
                                modifiers,
                            },
                            data: EventData {
                                rect: *rect,
                                target: node_id.clone(),
                            },
                            handle: handle.clone(),
                        });
                    }
                    if !stop_propagation {
                        stop_propagation = handle.get_stop_propagation();
                    }
                    if focus_set && stop_propagation {
                        return false;
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
                for handler in event {
                    handler.borrow_mut()(
                        val.clone(),
                        EventData {
                            rect,
                            target: node_id.clone(),
                        },
                        handle.clone(),
                    );
                }
            },
            AllowDisabled::Disallow,
        );
    }
}
