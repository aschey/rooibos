use std::cell::RefCell;
use std::rc::Rc;

use ratatui::layout::{Position, Rect};
use terminput::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};

use super::{EventData, MouseEventFn};
use crate::{
    focus_next, focus_prev, set_pending_resize, toggle_print_dom, with_nodes, with_nodes_mut,
    DomNodeKey,
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

struct ClickEvent {
    on_click: Option<MouseEventFn>,
    rect: Rect,
    key: DomNodeKey,
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
            Event::Key(key_event) => {
                if key_event.code == KeyCode::Tab && key_event.kind == KeyEventKind::Press {
                    focus_next();
                } else if key_event.code == KeyCode::BackTab
                    && key_event.kind == KeyEventKind::Press
                {
                    focus_prev();
                } else if key_event.code == KeyCode::Char('x')
                    && key_event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    toggle_print_dom();
                } else if let Some(key) = with_nodes(|nodes| nodes.focused_key()) {
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
                        let current: Rc<RefCell<Option<ClickEvent>>> = Rc::new(RefCell::new(None));
                        for root in nodes.roots_desc() {
                            let found = root.key().traverse(
                                |key, inner| {
                                    if inner.disabled {
                                        return None;
                                    }

                                    let rect = inner.rect.borrow();
                                    if rect.contains(Position {
                                        x: mouse_event.column,
                                        y: mouse_event.row,
                                    }) {
                                        if inner.focusable
                                            || inner.event_handlers.on_click.is_some()
                                        {
                                            *current.borrow_mut() = Some(ClickEvent {
                                                on_click: inner.event_handlers.on_click.clone(),
                                                rect: *rect,
                                                key,
                                            });
                                        }
                                        if inner.event_handlers.on_click.is_some() {
                                            return Some(());
                                        }
                                    }
                                    None
                                },
                                true,
                            );
                            if !found.is_empty() {
                                break;
                            }
                        }

                        current
                    });
                    let current = current.borrow();
                    if let Some(ClickEvent {
                        on_click,
                        rect,
                        key,
                    }) = current.as_ref()
                    {
                        with_nodes_mut(|nodes| {
                            let set_focus =
                                nodes.focused_key() != Some(*key) && nodes[*key].focusable;
                            if set_focus {
                                nodes.set_focused(*key);
                            }
                        });

                        if let Some(on_click) = on_click {
                            #[cfg(debug_assertions)]
                            let _guard =
                                reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                            on_click.borrow_mut()(mouse_event, EventData { rect: *rect });
                        }
                    }
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
                                true,
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
            },
            Event::Paste(val) => {
                if let Some(key) = with_nodes(|nodes| nodes.focused_key()) {
                    let (rect, on_paste) = with_nodes(|nodes| {
                        (
                            *nodes[key].rect.borrow(),
                            nodes[key].event_handlers.on_paste.clone(),
                        )
                    });
                    if let Some(on_paste) = on_paste {
                        #[cfg(debug_assertions)]
                        let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
                        on_paste.borrow_mut()(val, EventData { rect });
                    }
                }
            }
            Event::Resize(_, _) => {
                set_pending_resize();
            }
        };
    }
}
