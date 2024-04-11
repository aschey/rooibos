use std::any::Any;
use std::process::Output;

use any_spawner::Executor;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use dyn_clonable::clonable;
use futures_util::StreamExt;
use reactive_graph::computed::Memo;
use reactive_graph::effect::Effect;
use reactive_graph::owner::{provide_context, use_context, Owner};
use reactive_graph::signal::{signal, ReadSignal, WriteSignal};
use reactive_graph::traits::{Get, Set, UpdateUntracked};
use rooibos_dom::{focused_node, NodeId};
use tokio::sync::{mpsc, watch};
use tokio::task;

pub enum Event {
    TermEvent(crossterm::event::Event),
    CancellationComplete(Option<String>),
    QuitRequested,
}

async fn read_input(event_tx: mpsc::Sender<Event>) {
    let mut event_reader = crossterm::event::EventStream::new().fuse();
    while let Some(Ok(event)) = event_reader.next().await {
        if let event::Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                event_tx.send(Event::QuitRequested).await.unwrap();
                break;
            }
            event_tx.send(Event::TermEvent(event)).await.unwrap();
        }
    }
}

#[derive(Clone, Copy)]
struct TermSignal(ReadSignal<Option<crossterm::event::Event>>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TickResult {
    Continue,
    Exit,
}

#[clonable]
pub trait AnyClone: Any + Clone {
    fn as_any(&self) -> &dyn Any;
}

impl<T> AnyClone for T
where
    T: Any + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn execute<T>(f: impl FnOnce() -> T) -> T {
    let owner = Owner::new();
    let res = owner.with(f);
    drop(owner);
    res
}

pub async fn init(f: impl std::future::Future) {
    any_spawner::Executor::init_tokio().unwrap();
    let local = task::LocalSet::new();
    local
        .run_until(async move {
            f.await;
        })
        .await;
}

pub struct Runtime {
    event_rx: mpsc::Receiver<Event>,
    custom_event_tx: mpsc::Sender<Box<dyn AnyClone + Send>>,
    custom_event_rx: mpsc::Receiver<Box<dyn AnyClone + Send>>,
    dom_update_tx: watch::Sender<()>,
    dom_update_rx: watch::Receiver<()>,
    set_last_term_event: WriteSignal<Option<crossterm::event::Event>>,
}

impl Runtime {
    pub fn initialize() -> Self {
        let (event_tx, event_rx) = mpsc::channel(32);
        let (custom_event_tx, custom_event_rx) = mpsc::channel(32);
        let (dom_update_tx, dom_update_rx) = watch::channel(());
        let (last_term_event, set_last_term_event) = signal(None);
        provide_context(TermSignal(last_term_event));
        tokio::spawn(async move { read_input(event_tx).await });
        Self {
            event_rx,
            custom_event_tx,
            custom_event_rx,
            dom_update_tx,
            dom_update_rx,
            set_last_term_event,
        }
    }

    pub fn connect_update(&self) -> watch::Sender<()> {
        self.dom_update_tx.clone()
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle {
            custom_event_tx: self.custom_event_tx.clone(),
        }
    }

    pub async fn tick(&mut self) -> TickResult {
        loop {
            tokio::select! {
                event = self.event_rx.recv() => {
                    match event {
                        Some(Event::TermEvent(e)) => {
                            self.set_last_term_event.set(Some(e));
                            // return TickResult::Continue;
                        }
                        Some(Event::QuitRequested) => {
                            return TickResult::Exit;
                        }
                        _ => {}
                    }
                }
                custom_event = self.custom_event_rx.recv() => {

                }
                _ = self.dom_update_rx.changed() => {
                    // self.set_last_term_event.update_untracked(|v| *v = None);
                    return TickResult::Continue;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RuntimeHandle {
    custom_event_tx: mpsc::Sender<Box<dyn AnyClone + Send>>,
}

impl RuntimeHandle {
    pub async fn send_message<T: AnyClone + Send + Sync + 'static>(&self, message: T) {
        self.custom_event_tx.send(Box::new(message)).await.unwrap();
    }
}

pub fn key_effect<T>(f: impl Fn(KeyEvent) -> T + 'static) -> Effect {
    let last_term_event = use_context::<TermSignal>().unwrap();
    Effect::new(move |_| {
        if let Some(crossterm::event::Event::Key(key_event)) = last_term_event.0.get() {
            if key_event.kind == KeyEventKind::Press {
                f(key_event);
            }
        }
    })
}

pub fn use_focus() -> (NodeId, impl Get<Value = bool> + Copy) {
    let id = NodeId::new_auto();
    use_focus_with_id_inner(id)
}

pub fn use_focus_with_id(id: impl Into<String>) -> (NodeId, impl Get<Value = bool> + Copy) {
    let id = NodeId::new(id);
    use_focus_with_id_inner(id)
}

fn use_focus_with_id_inner(id: NodeId) -> (NodeId, impl Get<Value = bool> + Copy) {
    let focused_node = focused_node();
    let focused = Memo::new({
        let id = id.clone();
        move |_| focused_node.get().map(|node| node == id).unwrap_or(false)
    });

    (id, focused)
}
