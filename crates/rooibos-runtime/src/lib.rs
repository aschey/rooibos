use std::future::Future;
use std::io::{self, stdout, Stdout};
use std::panic::{set_hook, take_hook};
use std::sync::OnceLock;

use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures_util::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use reactive_graph::owner::Owner;
use reactive_graph::signal::{signal, ReadSignal};
use reactive_graph::traits::Set;
use rooibos_dom::{dom_update_receiver, render_dom, send_key, DomUpdateReceiver};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task;

static CURRENT_RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();
static TERM_TX: OnceLock<broadcast::Sender<rooibos_dom::KeyEvent>> = OnceLock::new();

async fn read_input(quit_tx: mpsc::Sender<()>, term_tx: broadcast::Sender<rooibos_dom::KeyEvent>) {
    let mut event_reader = crossterm::event::EventStream::new().fuse();
    while let Some(Ok(event)) = event_reader.next().await {
        if let event::Event::Key(key_event) = event {
            let KeyEvent {
                code, modifiers, ..
            } = key_event;
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                quit_tx.send(()).await.unwrap();
                break;
            }
            term_tx.send(key_event.into()).ok();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TickResult {
    Continue,
    Redraw,
    Exit,
}

pub fn execute<T>(f: impl FnOnce() -> T) -> T {
    let owner = Owner::new();
    set_panic_hook();
    let res = owner.with(f);
    drop(owner);
    restore_terminal().unwrap();
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
    quit_rx: mpsc::Receiver<()>,
    dom_update_rx: DomUpdateReceiver,
    term_event_rx: broadcast::Receiver<rooibos_dom::KeyEvent>,
}

impl Runtime {
    fn initialize() -> Self {
        let (quit_tx, quit_rx) = mpsc::channel(32);
        let (term_event_tx, term_event_rx) = broadcast::channel(32);
        TERM_TX.set(term_event_tx.clone()).unwrap();
        let dom_update_rx = dom_update_receiver();

        tokio::spawn(async move { read_input(quit_tx, term_event_tx).await });
        Self {
            dom_update_rx,
            quit_rx,
            term_event_rx,
        }
    }

    async fn tick(&mut self) -> TickResult {
        tokio::select! {
            _ = self.quit_rx.recv() => {
                TickResult::Exit
            }
            _ = self.dom_update_rx.changed() => {
                TickResult::Redraw
            }
            term_event = self.term_event_rx.recv() => {
                if let Ok(term_event) = term_event {
                    send_key(term_event)
                }
                TickResult::Continue
            }
        }
    }
}

pub async fn tick() -> TickResult {
    let rt = CURRENT_RUNTIME.get().unwrap();
    rt.lock().await.tick().await
}

pub fn use_keypress() -> ReadSignal<Option<rooibos_dom::KeyEvent>> {
    let mut term_rx = TERM_TX.get().unwrap().subscribe();
    let (term_signal, set_term_signal) = signal(None);
    tokio::spawn(async move {
        // TODO: this doesn't work?
        // if term_signal.is_disposed() {
        //     return;
        // }
        while let Ok(key_event) = term_rx.recv().await {
            if key_event.kind == rooibos_dom::KeyEventKind::Press {
                set_term_signal.set(Some(key_event));
            }
        }
    });

    term_signal
}

pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

pub fn restore_terminal() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn set_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}

pub async fn run() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    terminal.draw(render_dom)?;
    loop {
        let tick_result = tick().await;
        match tick_result {
            TickResult::Redraw => {
                terminal.draw(render_dom)?;
            }
            TickResult::Exit => {
                return Ok(());
            }
            TickResult::Continue => {}
        }
    }
}
