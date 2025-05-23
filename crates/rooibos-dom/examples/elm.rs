use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Stdout, stdout};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rooibos_dom::events::{KeyEventProps, dispatch_event};
use rooibos_dom::widgets::RenderWidgetRef;
use rooibos_dom::{
    AsDomNode, Borders, DomNode, DomWidgetNode, NodeId, NonblockingTerminal, dom_update_receiver,
    focus_next, mount, render_terminal, with_nodes, with_nodes_mut,
};
use taffy::style_helpers::length;
use terminput::{CTRL, Event, KeyCode, Repeats, key};
use terminput_crossterm::to_terminput;
use tokio::sync::mpsc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel(32);

    let mut app = Counters {
        counters: BTreeMap::new(),
        next_id: 0,
        focused: false,
    };
    app.update(Message::Add);
    let view = {
        let tx = tx.clone();
        app.view(move |msg| tx.clone().try_send(msg).unwrap())
    };

    let mut dom_update_rx = dom_update_receiver();
    let mut terminal = setup_terminal()?;
    mount(view);

    render_terminal(&mut terminal).await?;
    let mut event_reader = EventStream::new().fuse();
    focus_next();

    loop {
        tokio::select! {
            Ok(()) = dom_update_rx.changed() => {
                render_terminal(&mut terminal).await?;
            }
            Some(msg) = rx.recv() => {
                let prev_focused_id = with_nodes(|nodes| nodes.focused().clone());
                with_nodes_mut(|nodes| nodes.clear());
                app.update(msg);
                let tx = tx.clone();
                let view = app.view(move |msg| tx.clone().try_send(msg).unwrap());
                mount(view);

                with_nodes_mut(|nodes| {
                    let prev_focused_key = prev_focused_id.and_then(|p| nodes.get_key(p));
                    nodes.set_focused_untracked(prev_focused_key);
                });
            }
            Some(Ok(event)) = event_reader.next() => {
                if let Ok(event) = to_terminput(event) {
                    if should_exit(&event) {
                        break;
                    }

                    dispatch_event(event.into())
                }
            }
            else => {
                break;
            }
        }
    }

    restore_terminal()?;
    Ok(())
}

fn should_exit(event: &Event) -> bool {
    if let Some(key_event) = event.as_key_press(Repeats::Include) {
        matches!(key_event, key!(CTRL, KeyCode::Char('c')))
    } else {
        false
    }
}
fn setup_terminal() -> Result<NonblockingTerminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Hide, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(NonblockingTerminal::new(terminal))
}

fn restore_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen, Show)?;
    Ok(())
}

enum Message {
    Task {
        task_message: TaskMessage,
        id: usize,
    },
    Add,
    Focus,
    Blur,
}

struct Counters {
    counters: BTreeMap<usize, Counter>,
    next_id: usize,
    focused: bool,
}

impl Counters {
    fn update(&mut self, message: Message) {
        match message {
            Message::Task {
                task_message: TaskMessage::Delete,
                id,
            } => {
                self.counters.retain(|_, c| c.id != id);
            }
            Message::Task { task_message, id } => {
                self.counters.get_mut(&id).unwrap().update(task_message);
            }
            Message::Add => {
                self.counters.insert(
                    self.next_id,
                    Counter {
                        id: self.next_id,
                        count: 0,
                        focused: false,
                    },
                );
                self.next_id += 1;
            }
            Message::Focus => {
                self.focused = true;
            }
            Message::Blur => {
                self.focused = false;
            }
        }
    }

    fn view<F>(&self, send: F) -> impl AsDomNode + use<F>
    where
        F: Fn(Message) + Clone + 'static,
    {
        let id: NodeId = "counter_holder".into();
        let mut col = {
            let send = send.clone();
            DomNode::flex_col()
                .on_key_down({
                    let send = send.clone();
                    move |props: KeyEventProps| {
                        if props.event.code == KeyCode::Char('a') {
                            send(Message::Add);
                        }
                    }
                })
                .focusable(true)
                .id(id.clone())
                .on_direct_focus({
                    let send = send.clone();
                    move |_, _, _| send(Message::Focus)
                })
                .on_direct_blur(move |_, _, _| send(Message::Blur))
        };
        if self.focused && !self.counters.is_empty() {
            col = col.borders(Borders::all());
        }

        with_nodes_mut(|nodes| {
            nodes.update_layout(col.get_key(), |layout| {
                layout.padding = length(1.0);
            })
        });

        for (id, counter) in self.counters.iter() {
            let send = send.clone();
            let id = *id;
            col.append(&counter.view(move |task_message| send(Message::Task { id, task_message })));
        }
        col
    }
}

enum TaskMessage {
    Increment,
    Decrement,
    Focus,
    Blur,
    Delete,
}

#[derive(Clone)]
struct Counter {
    count: i32,
    id: usize,
    focused: bool,
}

impl Counter {
    fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Increment => {
                self.count += 1;
            }
            TaskMessage::Decrement => {
                self.count -= 1;
            }
            TaskMessage::Focus => {
                self.focused = true;
            }
            TaskMessage::Blur => {
                self.focused = false;
            }
            TaskMessage::Delete => {
                // handled by parent
            }
        }
    }

    fn view<F>(&self, send: F) -> impl AsDomNode
    where
        F: Fn(TaskMessage) + Clone + 'static,
    {
        let model = self.clone();
        let id: NodeId = format!("counter{}", self.id).into();
        let borders = if model.focused {
            Borders::all()
        } else {
            Borders::all().empty()
        };
        let widget = DomWidgetNode::new(move || RenderWidgetRef(format!("count: {}", model.count)));
        widget.build();

        let node = DomNode::widget(widget.clone())
            .on_key_down({
                let send = send.clone();
                move |props: KeyEventProps| match props.event.code {
                    KeyCode::Char('+') => {
                        send(TaskMessage::Increment);
                    }
                    KeyCode::Char('-') => {
                        send(TaskMessage::Decrement);
                    }
                    KeyCode::Char('d') => {
                        send(TaskMessage::Delete);
                    }
                    _ => {}
                }
            })
            .on_click({
                let send = send.clone();
                move |_| send(TaskMessage::Increment)
            })
            .on_right_click({
                let send = send.clone();
                move |_| send(TaskMessage::Decrement)
            })
            .id(id.clone())
            .focusable(true)
            .on_direct_focus({
                let send = send.clone();
                move |_, _, _| send(TaskMessage::Focus)
            })
            .on_direct_blur(move |_, _, _| send(TaskMessage::Blur));

        with_nodes_mut(|nodes| {
            let rect = borders.to_rect();
            nodes.set_borders(node.get_key(), borders);
            nodes.update_layout(node.get_key(), |layout| {
                layout.border = rect;
            });
        });

        node
    }
}
