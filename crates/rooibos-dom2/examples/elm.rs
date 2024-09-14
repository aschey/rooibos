use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::io::{stdout, Stdout};
use std::rc::Rc;
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Paragraph, Widget as _, WidgetRef};
use ratatui::Terminal;
use rooibos_dom2::{
    dispatch_event, dom_update_receiver, focus_next, render_dom, with_nodes, with_nodes_mut,
    AsDomNode, DomNode, DomWidgetNode,
};
use taffy::{Dimension, Size};
use terminput::{Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    let (tx, mut rx) = mpsc::channel(32);

    let mut app = Counters {
        counters: BTreeMap::new(),
        next_id: 0,
    };
    let tx_ = tx.clone();
    let view = app.view(move |msg| tx_.clone().try_send(msg).unwrap());
    with_nodes_mut(|nodes| nodes.set_root(0, view));

    terminal.draw(|frame| render_dom(frame.buffer_mut()))?;

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
                with_nodes_mut(|nodes| {
                    nodes.set_root(0, view);
                    let prev_focused_key = prev_focused_id.and_then(|p| nodes.get_key(p));
                    nodes.set_focused(prev_focused_key);
                });

                terminal.draw(|frame| render_dom(frame.buffer_mut()))?;
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
}

struct Counters {
    counters: BTreeMap<usize, Counter>,
    next_id: usize,
}

impl Counters {
    fn update(&mut self, message: Message) {
        match message {
            Message::Task { task_message, id } => {
                self.counters.get_mut(&id).unwrap().update(task_message);
            }
            Message::Add => {
                self.counters.insert(
                    self.next_id,
                    Counter {
                        id: self.next_id,
                        count: 0,
                    },
                );
                self.next_id += 1;
            }
        }
    }

    fn view<F>(&self, send: F) -> impl AsDomNode
    where
        F: Fn(Message) + Clone + 'static,
    {
        let col = {
            let send = send.clone();
            DomNode::flex_col().on_key_down(move |event, _, _| {
                if let KeyEvent {
                    code: KeyCode::Char('a'),
                    ..
                } = event
                {
                    send(Message::Add);
                }
            })
        };
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
}

#[derive(Clone)]
struct Counter {
    count: i32,
    id: usize,
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
        }
    }

    fn view<F>(&self, send: F) -> impl AsDomNode
    where
        F: Fn(TaskMessage) + 'static,
    {
        let model = self.clone();

        let widget = DomWidgetNode::new::<Paragraph, _, _>(move || {
            let paragraph = Paragraph::new(format!("count: {}", model.count));
            move |rect, buf| paragraph.render_ref(rect, buf)
        });

        let node = DomNode::widget(widget.clone())
            .on_key_down(move |event, _, _| send(TaskMessage::Increment))
            .id(format!("counter{}", self.id));

        with_nodes_mut(|nodes| {
            nodes.update_layout(node.key(), |layout| {
                layout.size = Size {
                    width: Dimension::Length(10.),
                    height: Dimension::Length(1.),
                };
            });
        });

        node
    }
}
