use std::future::Future;
use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::panic::{set_hook, take_hook};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use any_spawner::Executor;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::queue;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use futures_util::StreamExt;
use ratatui::backend::CrosstermBackend;
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
use tracing::{error, warn};

type RestoreFn = dyn Fn() -> io::Result<()> + Send;
static CURRENT_RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();
static TERM_TX: OnceLock<broadcast::Sender<rooibos_dom::Event>> = OnceLock::new();
static SUPPORTS_KEYBOARD_ENHANCEMENT: AtomicBool = AtomicBool::new(false);
static RESTORE_TERMINAL: OnceLock<std::sync::Mutex<Box<RestoreFn>>> = OnceLock::new();

async fn read_input(quit_tx: mpsc::Sender<()>, term_tx: broadcast::Sender<rooibos_dom::Event>) {
    let mut event_reader = crossterm::event::EventStream::new().fuse();
    while let Some(Ok(event)) = event_reader.next().await {
        if let event::Event::Key(key_event) = event {
            let KeyEvent {
                code, modifiers, ..
            } = key_event;

            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                let _ = quit_tx
                    .send(())
                    .await
                    .tap_err(|e| warn!("error sending quit signal {e:?}"));
                break;
            }
        }
        term_tx.send(event.into()).ok();
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

pub fn init(settings: RuntimeSettings) {
    CURRENT_RUNTIME
        .set(Mutex::new(Runtime::initialize(settings)))
        .expect("init called more than once");
}

pub fn start<F, M>(settings: RuntimeSettings, f: F)
where
    F: FnOnce() -> M + 'static,
    M: Render,
{
    init(settings);
    mount(f);
}

#[derive(Debug)]
struct Runtime {
    quit_rx: mpsc::Receiver<()>,
    dom_update_rx: DomUpdateReceiver,
    term_event_rx: broadcast::Receiver<rooibos_dom::Event>,
}

impl Runtime {
    fn initialize(settings: RuntimeSettings) -> Self {
        let (quit_tx, quit_rx) = mpsc::channel(32);
        let (term_event_tx, term_event_rx) = broadcast::channel(32);
        TERM_TX
            .set(term_event_tx.clone())
            .expect("runtime initialized more than once");
        let dom_update_rx = dom_update_receiver();

        // We need to query this info before reading events from crossterm
        SUPPORTS_KEYBOARD_ENHANCEMENT.store(
            supports_keyboard_enhancement().unwrap_or(false),
            Ordering::Relaxed,
        );
        if settings.enable_input_reader {
            Executor::spawn(async move { read_input(quit_tx, term_event_tx).await });
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

pub fn setup_terminal<W>(settings: TerminalSettings<W>) -> io::Result<Terminal<CrosstermBackend<W>>>
where
    W: Write + 'static,
{
    let mut writer = (settings.get_writer)();
    enable_raw_mode()?;
    if settings.alternate_screen {
        queue!(writer, EnterAlternateScreen)?;
    }
    if settings.mouse_capture {
        queue!(writer, EnableMouseCapture)?;
    }
    if settings.keyboard_enhancement {
        queue!(
            writer,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
        )?;
    } else {
        SUPPORTS_KEYBOARD_ENHANCEMENT.store(false, Ordering::Relaxed);
    }
    writer.flush()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(writer))?;
    terminal.clear()?;

    *RESTORE_TERMINAL
        .get_or_init(|| std::sync::Mutex::new(Box::new(|| Ok(()))))
        .lock()
        .expect("lock poisoned") = Box::new(move || {
        let mut writer = (settings.get_writer)();

        if settings.keyboard_enhancement {
            queue!(writer, PopKeyboardEnhancementFlags)?;
        }

        if settings.mouse_capture {
            queue!(writer, DisableMouseCapture)?;
        }

        if settings.alternate_screen {
            queue!(writer, LeaveAlternateScreen)?;
        }
        writer.flush()?;
        disable_raw_mode()?;

        Ok::<_, io::Error>(())
    });
    Ok(terminal)
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

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    get_writer: Box<dyn Fn() -> W + Send>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            get_writer: Box::new(stdout),
        }
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            get_writer: Box::new(stderr),
        }
    }
}

impl<W> TerminalSettings<W> {
    pub fn alternate_screen(mut self, alternate_screen: bool) -> Self {
        self.alternate_screen = alternate_screen;
        self
    }

    pub fn mouse_capture(mut self, mouse_capture: bool) -> Self {
        self.mouse_capture = mouse_capture;
        self
    }

    pub fn keyboard_enhancement(mut self, keyboard_enhancement: bool) -> Self {
        self.keyboard_enhancement = keyboard_enhancement;
        self
    }

    pub fn writer(mut self, get_writer: impl Fn() -> W + Send + 'static) -> Self {
        self.get_writer = Box::new(get_writer);
        self
    }
}

pub async fn run<W>(settings: TerminalSettings<W>) -> io::Result<()>
where
    W: Write + 'static,
{
    let mut terminal = setup_terminal(settings)?;
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

pub fn delay<F>(duration: Duration, f: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_local(async move {
        tokio::time::sleep(duration).await;
        f.await;
    });
}
