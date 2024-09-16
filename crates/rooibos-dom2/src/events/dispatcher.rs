use std::cell::RefCell;

use ratatui::layout::{Position, Rect};
use terminput::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::EventData;
use crate::{
    focus_next, focus_prev, set_pending_resize, toggle_print_dom, with_nodes, with_nodes_mut,
    DomNodeKey, EventHandle, EventHandlers, MatchBehavior,
};

thread_local! {
    static EVENT_DISPATCHER: RefCell<EventDispatcher> = RefCell::new(EventDispatcher::new())
}

struct EventDispatcher {
    last_mouse_position: MouseEvent,
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
            last_mouse_position: MouseEvent {
                kind: MouseEventKind::Moved,
                column: u16::MAX,
                row: u16::MAX,
                modifiers: KeyModifiers::empty(),
            },
        }
    }

    fn reset_mouse_position(&mut self) {
        self.dispatch(Event::Mouse(self.last_mouse_position))
    }

    fn dispatch(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => dispatch_key_event(key_event),
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Mouse(mouse_event) => self.dispatch_mouse_event(mouse_event),
            Event::Paste(val) => dispatch_paste(val),
            Event::Resize(_, _) => {
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
                self.last_mouse_position = mouse_event;
                let current = with_nodes(|nodes| {
                    for root in nodes.roots_desc() {
                        let found = root.key().traverse(
                            |key, inner| {
                                if inner.disabled {
                                    return None;
                                }

                                if inner.rect.borrow().contains(Position {
                                    x: mouse_event.column,
                                    y: mouse_event.row,
                                }) && inner.focusable
                                {
                                    Some(key)
                                } else {
                                    None
                                }
                            },
                            MatchBehavior::SearchChildrenOnMatch,
                        );
                        if let Some(found) = found.first() {
                            return Some(*found);
                        }
                    }
                    None
                });

                if let Some(current) = current {
                    with_nodes_mut(|nodes| {
                        let set_focus = nodes.hovered_key() != Some(current);
                        if set_focus {
                            nodes.set_hovered(current);
                        }
                    });
                } else {
                    with_nodes_mut(|nodes| {
                        nodes.remove_hovered();
                    });
                }
            }
            MouseEventKind::ScrollDown => {}
            MouseEventKind::ScrollUp => {}
            MouseEventKind::ScrollLeft => {}
            MouseEventKind::ScrollRight => {}
        }
    }
}

fn dispatch_key_event(key_event: KeyEvent) {
    if key_event.code == KeyCode::Tab && key_event.kind == KeyEventKind::Press {
        focus_next();
    } else if key_event.code == KeyCode::BackTab && key_event.kind == KeyEventKind::Press {
        focus_prev();
    } else if key_event.code == KeyCode::Char('x')
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
    {
        toggle_print_dom();
    } else if let Some(key) = with_nodes(|nodes| nodes.focused_key()) {
        match key_event.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => {
                bubble_key_event(
                    key,
                    |handlers| handlers.on_key_down.clone(),
                    |event, rect, handle| {
                        event.borrow_mut()(key_event, EventData { rect }, handle);
                    },
                );
            }
            KeyEventKind::Release => {
                bubble_key_event(
                    key,
                    |handlers| handlers.on_key_up.clone(),
                    |event, rect, handle| {
                        event.borrow_mut()(key_event, EventData { rect }, handle);
                    },
                );
            }
        }
    }
}

fn bubble_key_event<GE, EF, E>(key: DomNodeKey, get_event: GE, event_fn: EF)
where
    GE: Fn(&EventHandlers) -> Option<E>,
    EF: Fn(&mut E, Rect, &mut EventHandle),
{
    let (rect, event) = with_nodes(|nodes| {
        (
            *nodes[key].rect.borrow(),
            get_event(&nodes[key].event_handlers),
        )
    });
    if let Some(mut event) = event {
        let mut handle = EventHandle::default();
        event_fn(&mut event, rect, &mut handle);
        if handle.stop_propagation {
            return;
        }
    }
    if let Some(parent) = with_nodes(|nodes| nodes[key].parent) {
        bubble_key_event(parent, get_event, event_fn)
    }
}

fn hit_test(position: Position) -> Vec<DomNodeKey> {
    let roots = with_nodes(|nodes| nodes.roots_desc());
    for root in roots {
        let found = root.key().traverse(
            |key, inner| {
                if inner.disabled {
                    return None;
                }

                let rect = inner.rect.borrow();
                if rect.contains(position) {
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
    let found = hit_test(Position {
        x: mouse_event.column,
        y: mouse_event.row,
    });
    if !found.is_empty() {
        let mut focus_set = false;
        let mut stop_propagation = false;
        for key in found.into_iter().rev() {
            let continue_iter = with_nodes_mut(|nodes| {
                if !focus_set && nodes[key].focusable {
                    focus_set = true;
                    if nodes.focused_key() != Some(key) {
                        nodes.set_focused(Some(key));
                    }
                    if stop_propagation {
                        return false;
                    }
                }
                if !stop_propagation {
                    if let Some(on_click) = &nodes[key].event_handlers.on_click {
                        let mut handle = EventHandle::default();
                        let rect = nodes[key].rect.borrow();
                        on_click.borrow_mut()(
                            crate::ClickEvent {
                                column: mouse_event.column,
                                row: mouse_event.row,
                                mouse_button,
                                modifiers: mouse_event.modifiers,
                            },
                            EventData { rect: *rect },
                            &mut handle,
                        );
                        stop_propagation = handle.stop_propagation;
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
        let (rect, on_paste) = with_nodes(|nodes| {
            (
                *nodes[key].rect.borrow(),
                nodes[key].event_handlers.on_paste.clone(),
            )
        });
        if let Some(on_paste) = on_paste {
            on_paste.borrow_mut()(val, EventData { rect }, &mut EventHandle::default());
        }
    }
}
