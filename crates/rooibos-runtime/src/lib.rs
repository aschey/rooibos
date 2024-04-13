use std::any::Any;
use std::cell::OnceCell;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use any_spawner::Executor;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures_util::StreamExt;
use reactive_graph::computed::Memo;
use reactive_graph::effect::Effect;
use reactive_graph::owner::{provide_context, use_context, Owner};
use reactive_graph::signal::{signal, ReadSignal, WriteSignal};
use reactive_graph::traits::{Get, Set, UpdateUntracked};
use rooibos_dom::{dom_update_receiver, focused_node, DomUpdateReceiver, NodeId};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task;

static CURRENT_RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();

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

pub fn execute<T>(f: impl FnOnce() -> T) -> T {
    let owner = Owner::new();
    let res = owner.with(f);
    drop(owner);
    res
}

pub async fn init<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    any_spawner::Executor::init_tokio().unwrap();
    CURRENT_RUNTIME
        .set(Mutex::new(Runtime::initialize()))
        .unwrap();

    let local = task::LocalSet::new();
    local.run_until(f).await
}

#[derive(Debug)]
struct Runtime {
    event_rx: mpsc::Receiver<Event>,
    dom_update_rx: DomUpdateReceiver,
    set_last_term_event: WriteSignal<Option<crossterm::event::Event>>,
}

impl Runtime {
    fn initialize() -> Self {
        let (event_tx, event_rx) = mpsc::channel(32);
        let (last_term_event, set_last_term_event) = signal(None);
        let dom_update_rx = dom_update_receiver();
        provide_context(TermSignal(last_term_event));
        tokio::spawn(async move { read_input(event_tx).await });
        Self {
            event_rx,
            dom_update_rx,
            set_last_term_event,
        }
    }

    async fn tick(&mut self) -> TickResult {
        loop {
            tokio::select! {
                event = self.event_rx.recv() => {
                    match event {
                        Some(Event::TermEvent(e)) => {
                            self.set_last_term_event.set(Some(e));
                        }
                        Some(Event::QuitRequested) => {
                            return TickResult::Exit;
                        }
                        _ => {}
                    }
                }
                _ = self.dom_update_rx.changed() => {
                    return TickResult::Continue;
                }
            }
        }
    }
}

pub async fn tick() -> TickResult {
    let rt = CURRENT_RUNTIME.get().unwrap();
    rt.lock().await.tick().await
}

pub fn key_effect<T>(f: impl Fn(KeyEvent) -> T + 'static) -> Effect {
    let last_term_event = use_context::<TermSignal>().unwrap();
    // prevent key events from firing on mount
    // TODO: is there a better way to do this? probably
    let init = AtomicBool::new(false);
    Effect::new(move |_| {
        let is_init = init.swap(true, Ordering::Relaxed);
        if let Some(crossterm::event::Event::Key(key_event)) = last_term_event.0.get() {
            if !is_init {
                return;
            }
            if key_event.kind == KeyEventKind::Press {
                f(key_event);
            }
        }
    })
}
