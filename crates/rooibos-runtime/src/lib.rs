pub mod backend;

use std::future::Future;
use std::io;
use std::panic::{set_hook, take_hook};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use any_spawner::Executor;
use backend::Backend;
use ratatui::Terminal;
use reactive_graph::owner::Owner;
use reactive_graph::signal::{signal, ReadSignal};
use reactive_graph::traits::Set;
use rooibos_dom::{
    dom_update_receiver, mount, render_dom, send_event, DomUpdateReceiver, Event, Render,
};
use tap::TapFallible;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task::{self, spawn_local};
use tracing::error;

type RestoreFn = dyn Fn() -> io::Result<()> + Send;

static CURRENT_RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();
static TERM_TX: OnceLock<broadcast::Sender<rooibos_dom::Event>> = OnceLock::new();
static SUPPORTS_KEYBOARD_ENHANCEMENT: AtomicBool = AtomicBool::new(false);
static RESTORE_TERMINAL: OnceLock<std::sync::Mutex<Box<RestoreFn>>> = OnceLock::new();

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
    let _ = restore_terminal().tap_err(|e| error!("error restoring terminal: {e:?}"));
    res
}

pub async fn init_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    any_spawner::Executor::init_tokio().expect("executor already initialized");

    let local = task::LocalSet::new();
    local.run_until(f).await
}

pub struct RuntimeSettings {
    enable_input_reader: bool,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            enable_input_reader: true,
        }
    }
}

impl RuntimeSettings {
    pub fn enable_input_reader(mut self, enable: bool) -> Self {
        self.enable_input_reader = enable;
        self
    }
}

pub fn init<B: Backend>(settings: RuntimeSettings, backend: &B) {
    CURRENT_RUNTIME
        .set(Mutex::new(Runtime::initialize(settings, backend)))
        .expect("init called more than once");
}

pub fn start<F, M, B>(settings: RuntimeSettings, backend: B, f: F) -> RuntimeHandle<B>
where
    B: Backend,
    F: FnOnce() -> M + 'static,
    M: Render,
{
    init(settings, &backend);
    mount(f);
    RuntimeHandle { backend }
}

#[derive(Debug)]
struct Runtime {
    quit_rx: mpsc::Receiver<()>,
    dom_update_rx: DomUpdateReceiver,
    term_event_rx: broadcast::Receiver<rooibos_dom::Event>,
}

impl Runtime {
    fn initialize<B: Backend>(settings: RuntimeSettings, backend: &B) -> Self {
        let (quit_tx, quit_rx) = mpsc::channel(32);
        let (term_event_tx, term_event_rx) = broadcast::channel(32);
        TERM_TX
            .set(term_event_tx.clone())
            .expect("runtime initialized more than once");
        let dom_update_rx = dom_update_receiver();

        // We need to query this info before reading events
        SUPPORTS_KEYBOARD_ENHANCEMENT
            .store(backend.supports_keyboard_enhancement(), Ordering::Relaxed);
        if settings.enable_input_reader {
            Executor::spawn(async move { B::read_input(quit_tx, term_event_tx).await });
        }

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
                    send_event(term_event)
                }
                TickResult::Continue
            }
        }
    }
}

pub async fn tick() -> TickResult {
    let rt = CURRENT_RUNTIME.get().expect("runtime not initialized");
    rt.lock().await.tick().await
}

pub fn use_keypress() -> ReadSignal<Option<rooibos_dom::KeyEvent>> {
    let mut term_rx = TERM_TX.get().expect("runtime not initialized").subscribe();
    let (term_signal, set_term_signal) = signal(None);
    Executor::spawn(async move {
        // TODO: this doesn't work?
        // if term_signal.is_disposed() {
        //     return;
        // }
        while let Ok(event) = term_rx.recv().await {
            if let Event::Key(key_event) = event {
                if key_event.kind == rooibos_dom::KeyEventKind::Press {
                    set_term_signal.set(Some(key_event));
                }
            }
        }
    });

    term_signal
}

pub fn supports_key_up() -> bool {
    SUPPORTS_KEYBOARD_ENHANCEMENT.load(Ordering::Relaxed)
}

pub fn restore_terminal() -> io::Result<()> {
    if let Some(restore) = RESTORE_TERMINAL.get() {
        restore.lock().expect("lock poisoned")()?;
    }
    Ok(())
}

pub fn set_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}

pub struct RuntimeHandle<B>
where
    B: Backend,
{
    backend: B,
}

impl<B> RuntimeHandle<B>
where
    B: Backend + 'static,
{
    pub fn setup_terminal(self) -> io::Result<Terminal<B::TuiBackend>> {
        let terminal = self.backend.setup_terminal()?;

        *RESTORE_TERMINAL
            .get_or_init(|| std::sync::Mutex::new(Box::new(|| Ok(()))))
            .lock()
            .expect("lock poisoned") = Box::new(move || self.backend.restore_terminal());

        Ok(terminal)
    }

    pub async fn run(self) -> io::Result<()> {
        let mut terminal = self.setup_terminal()?;
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
}

pub fn delay<F>(duration: Duration, f: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_local(async move {
        tokio::time::sleep(duration).await;
        f.await;
    });
}
