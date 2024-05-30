pub mod backend;

use std::future::Future;
use std::io;
use std::panic::{set_hook, take_hook};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use any_spawner::Executor;
use async_signal::{Signal, Signals};
use backend::Backend;
use futures_util::StreamExt;
use ratatui::Terminal;
use reactive_graph::owner::Owner;
use reactive_graph::signal::{signal, ReadSignal};
use reactive_graph::traits::Set;
use rooibos_dom::{
    dom_update_receiver, focus_next, mount, render_dom, send_event, DomUpdateReceiver, Event,
    Render,
};
use tap::TapFallible;
use tokio::sync::{broadcast, mpsc};
use tokio::task::{self, spawn_local};
use tracing::error;

type RestoreFn = dyn Fn() -> io::Result<()> + Send;

static TERM_TX: OnceLock<broadcast::Sender<rooibos_dom::Event>> = OnceLock::new();
static SUPPORTS_KEYBOARD_ENHANCEMENT: AtomicBool = AtomicBool::new(false);
static RESTORE_TERMINAL: OnceLock<std::sync::Mutex<Box<RestoreFn>>> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TickResult {
    Continue,
    Redraw,
    Restart,
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

#[derive(Debug)]
pub struct RuntimeSettings {
    enable_input_reader: bool,
    show_final_output: bool,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            enable_input_reader: true,
            show_final_output: true,
        }
    }
}

impl RuntimeSettings {
    pub fn enable_input_reader(mut self, enable: bool) -> Self {
        self.enable_input_reader = enable;
        self
    }

    pub fn show_final_output(mut self, show_final_output: bool) -> Self {
        self.show_final_output = show_final_output;
        self
    }
}

#[derive(Debug)]
pub struct Runtime<B: Backend> {
    settings: RuntimeSettings,
    signal_tx: mpsc::Sender<SignalMode>,
    signal_rx: mpsc::Receiver<SignalMode>,
    term_event_tx: broadcast::Sender<Event>,
    dom_update_rx: DomUpdateReceiver,
    term_event_rx: broadcast::Receiver<rooibos_dom::Event>,
    backend: Arc<B>,
}

pub enum SignalMode {
    Terminate,
    Suspend,
    Resume,
}

impl<B: Backend + 'static> Runtime<B> {
    pub fn initialize<F, M>(settings: RuntimeSettings, backend: B, f: F) -> Self
    where
        F: FnOnce() -> M + 'static,
        M: Render,
    {
        let backend = Arc::new(backend);
        let (signal_tx, signal_rx) = mpsc::channel(32);
        let (term_event_tx, term_event_rx) = broadcast::channel(32);
        TERM_TX
            .set(term_event_tx.clone())
            .expect("runtime initialized more than once");
        let dom_update_rx = dom_update_receiver();

        // We need to query this info before reading events
        SUPPORTS_KEYBOARD_ENHANCEMENT
            .store(backend.supports_keyboard_enhancement(), Ordering::Relaxed);

        {
            let signal_tx = signal_tx.clone();
            Executor::spawn(async move {
                #[cfg(unix)]
                // SIGSTP cannot be handled https://www.gnu.org/software/libc/manual/html_node/Job-Control-Signals.html
                let mut signals = Signals::new([
                    Signal::Term,
                    Signal::Quit,
                    Signal::Int,
                    Signal::Tstp,
                    Signal::Cont,
                ])
                .unwrap();

                #[cfg(windows)]
                let mut signals = Signals::new([Signal::Int]).unwrap();

                while let Some(Ok(signal)) = signals.next().await {
                    match signal {
                        Signal::Tstp => {
                            let _ = signal_tx.send(SignalMode::Suspend).await;
                        }
                        Signal::Cont => {
                            let _ = signal_tx.send(SignalMode::Resume).await;
                        }
                        _ => {
                            let _ = signal_tx.send(SignalMode::Terminate).await;
                        }
                    }
                }
            });
        }

        mount(f);
        Self {
            term_event_tx,
            signal_tx,
            settings,
            dom_update_rx,
            signal_rx,
            term_event_rx,
            backend,
        }
    }

    pub fn setup_terminal(&self) -> io::Result<Terminal<B::TuiBackend>> {
        let terminal = self.backend.setup_terminal()?;
        let backend = self.backend.clone();

        if self.settings.enable_input_reader {
            let backend = backend.clone();
            let signal_tx = self.signal_tx.clone();
            let term_event_tx = self.term_event_tx.clone();
            Executor::spawn(async move {
                backend.read_input(signal_tx, term_event_tx).await;
            });
        }
        let show_final_output = self.settings.show_final_output;
        *RESTORE_TERMINAL
            .get_or_init(|| std::sync::Mutex::new(Box::new(|| Ok(()))))
            .lock()
            .expect("lock poisoned") = Box::new(move || {
            backend.restore_terminal()?;
            if show_final_output {
                backend.write_all(b"\n")?;
            }
            Ok(())
        });

        Ok(terminal)
    }

    pub async fn run(mut self) -> io::Result<()> {
        let mut terminal = self.setup_terminal()?;
        terminal.draw(render_dom)?;
        focus_next();
        loop {
            let tick_result = self.tick().await;
            match tick_result {
                TickResult::Redraw => {
                    terminal.draw(render_dom)?;
                }
                TickResult::Restart => {
                    terminal = self.setup_terminal()?;
                    terminal.draw(render_dom)?;
                }
                TickResult::Exit => {
                    if !self.settings.show_final_output {
                        terminal.clear()?;
                    }
                    return Ok(());
                }
                TickResult::Continue => {}
            }
        }
    }

    pub async fn tick(&mut self) -> TickResult {
        tokio::select! {
            signal = self.signal_rx.recv() => {
                match signal {
                    Some(SignalMode::Suspend) => {
                        restore_terminal().unwrap();
                        #[cfg(unix)]
                        signal_hook::low_level::emulate_default_handler(Signal::Tstp as i32).unwrap();
                        TickResult::Continue
                    }
                    Some(SignalMode::Resume) => {
                        #[cfg(unix)]
                        signal_hook::low_level::emulate_default_handler(Signal::Cont as i32).unwrap();
                        TickResult::Restart
                    }
                    Some(SignalMode::Terminate) | None => {
                        TickResult::Exit
                    }
                }
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

pub fn delay<F>(duration: Duration, f: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_local(async move {
        tokio::time::sleep(duration).await;
        f.await;
    });
}
