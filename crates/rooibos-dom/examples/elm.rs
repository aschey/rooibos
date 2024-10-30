use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Stdout, stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, WidgetRef};
use rooibos_dom::events::{KeyEventProps, dispatch_event};
use rooibos_dom::{
    AsDomNode, DomNode, DomWidgetNode, NodeId, focus_next, mount, render_terminal, with_nodes,
    with_nodes_mut,
};
use taffy::style_helpers::length;
use taffy::{Dimension, Size};
use terminput::{Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    let (tx, mut rx) = mpsc::channel(32);

    let mut app = Counters {
        counters: BTreeMap::new(),
        next_id: 0,
        focused: false,
    };
    let view = {
        let tx = tx.clone();
        app.view(move |msg| tx.clone().try_send(msg).unwrap())
    };
    mount(view);

    render_terminal(&mut terminal)?;
    let mut event_reader = EventStream::new().fuse();
    focus_next();

    loop {
        tokio::select! {
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
                render_terminal(&mut terminal)?;
            }
            Some(Ok(event)) = event_reader.next() => {
                if let Ok(event) = event.try_into() {
                    if should_exit(&event) {
                        break;
                    }

                    dispatch_event(event)
                }
            }
            else => {
                break;
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn should_exit(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })
    )
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
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
                self.counters.insert(self.next_id, Counter {
                    id: self.next_id,
                    count: 0,
                    focused: false,
                });
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

    fn view<F>(&self, send: F) -> impl AsDomNode
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
                .on_focus({
                    let send = send.clone();
                    move |_, _| send(Message::Focus)
                })
                .on_blur(move |_, _| send(Message::Blur))
        };
        if self.focused {
            col = col.block(Block::bordered());
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

        let widget = DomWidgetNode::new::<Paragraph, _>(move || {
            let mut paragraph = Paragraph::new(format!("count: {}", model.count));
            if model.focused {
                paragraph = paragraph.block(Block::bordered());
            } else {
                paragraph = paragraph.block(Block::bordered().border_set(border::EMPTY))
            }

            move |rect, frame| paragraph.render_ref(rect, frame.buffer_mut())
        });
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
            .on_focus({
                let send = send.clone();
                move |_, _| send(TaskMessage::Focus)
            })
            .on_blur(move |_, _| send(TaskMessage::Blur));

        with_nodes_mut(|nodes| {
            nodes.update_layout(node.get_key(), |layout| {
                layout.size = Size {
                    width: Dimension::Length(15.),
                    height: Dimension::Length(3.),
                };
            });
        });

        node
    }
}
