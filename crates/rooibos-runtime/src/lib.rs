pub mod backend;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::Display;
use std::future::Future;
use std::io;
use std::panic::{set_hook, take_hook};
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use backend::Backend;
use derivative::Derivative;
use futures_util::StreamExt;
use ratatui::backend::Backend as TuiBackend;
use ratatui::layout::Size;
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Terminal;
use reactive_graph::owner::Owner;
use reactive_graph::signal::{signal, ReadSignal};
use reactive_graph::traits::Set;
use rooibos_dom::{
    dom_update_receiver, focus_next, mount, render_dom, send_event, unmount, DomUpdateReceiver,
    Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, Render,
};
use tap::TapFallible;
use tokio::sync::broadcast;
use tokio::{task, task_local};
pub use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

pub mod wasm_compat {
    pub use ::wasm_compat::cell::*;
    pub use ::wasm_compat::futures::*;
    pub use ::wasm_compat::once::*;
    pub use ::wasm_compat::static_init;
    pub use ::wasm_compat::static_init::*;
    pub use ::wasm_compat::sync::*;
    pub use ::wasm_compat::time::*;
}

type RestoreFn = dyn Fn() -> io::Result<()> + Send;

struct RuntimeState {
    term_tx: broadcast::Sender<rooibos_dom::Event>,
    term_command_tx: broadcast::Sender<TerminalCommand>,
    runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    supports_keyboard_enhancement: bool,
    pixel_size: Option<Size>,
    restore_terminal: wasm_compat::Mutex<Box<RestoreFn>>,
}

impl RuntimeState {
    fn new() -> Self {
        let (term_tx, _) = broadcast::channel(32);
        let (term_command_tx, _) = broadcast::channel(32);
        let (runtime_command_tx, _) = broadcast::channel(32);
        Self {
            term_tx,
            term_command_tx,
            runtime_command_tx,
            supports_keyboard_enhancement: false,
            pixel_size: None,
            restore_terminal: wasm_compat::Mutex::new(Box::new(|| Ok(()))),
        }
    }
}

wasm_compat::static_init! {
    static STATE: wasm_compat::Once<wasm_compat::RwLock<HashMap<u32, RuntimeState>>> = wasm_compat::Once::new();
}

task_local! {
    static CURRENT_RUNTIME: u32;
}

pub async fn with_runtime<Fut, T>(id: u32, f: Fut) -> T
where
    Fut: Future<Output = T>,
{
    STATE.with(|s| {
        s.get()
            .unwrap()
            .borrow_mut()
            .write()
            .insert(id, RuntimeState::new())
    });

    CURRENT_RUNTIME.scope(id, f).await
}

#[cfg(not(target_arch = "wasm32"))]
pub type OnFinishFn = dyn FnOnce(ExitStatus, Option<tokio::process::ChildStdout>, Option<tokio::process::ChildStderr>)
    + Send
    + Sync;

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub enum TerminalCommand {
    InsertBefore {
        height: u16,
        text: Text<'static>,
    },
    EnterAltScreen,
    LeaveAltScreen,
    SetTitle(String),
    #[cfg(feature = "clipboard")]
    SetClipboard(String, backend::ClipboardKind),
    #[cfg(not(target_arch = "wasm32"))]
    Exec {
        #[derivative(Debug = "ignore")]
        command: Arc<std::sync::Mutex<tokio::process::Command>>,
        #[derivative(Debug = "ignore")]
        on_finish: Arc<std::sync::Mutex<Option<Box<OnFinishFn>>>>,
    },
    Poll,
}

#[derive(Clone)]
pub enum TickResult {
    Continue,
    Redraw,
    Restart,
    Command(TerminalCommand),
    Exit,
}

pub fn execute<T>(f: impl FnOnce() -> T) -> T {
    let mut state = HashMap::new();
    state.insert(0, RuntimeState::new());
    if STATE
        .with(|s| s.set(wasm_compat::RwLock::new(state)))
        .is_err()
    {
        panic!();
    }
    let owner = Owner::new();
    set_panic_hook(owner.clone());
    let res = owner.with(f);

    owner.cleanup();
    drop(owner);

    let _ = restore_terminal().tap_err(|e| error!("error restoring terminal: {e:?}"));
    res
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn init_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    any_spawner::Executor::init_tokio().expect("executor already initialized");
    let local = task::LocalSet::new();
    local.run_until(f).await
}

#[cfg(target_arch = "wasm32")]
pub async fn init_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    any_spawner::Executor::init_wasm_bindgen().expect("executor already initialized");
    f.await
}

#[derive(Debug)]
pub struct RuntimeSettings {
    enable_input_reader: bool,
    enable_signal_handler: bool,
    show_final_output: bool,
    hover_debounce: Duration,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            enable_input_reader: true,
            enable_signal_handler: true,
            show_final_output: true,
            hover_debounce: Duration::from_millis(20),
        }
    }
}

impl RuntimeSettings {
    pub fn enable_input_reader(mut self, enable: bool) -> Self {
        self.enable_input_reader = enable;
        self
    }

    pub fn enable_signal_handler(mut self, enable: bool) -> Self {
        self.enable_signal_handler = enable;
        self
    }

    pub fn show_final_output(mut self, show_final_output: bool) -> Self {
        self.show_final_output = show_final_output;
        self
    }

    pub fn hover_debounce(mut self, hover_debounce: Duration) -> Self {
        self.hover_debounce = hover_debounce;
        self
    }
}

#[derive(Debug)]
pub struct Runtime<B: Backend> {
    settings: RuntimeSettings,
    runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    runtime_command_rx: broadcast::Receiver<RuntimeCommand>,
    term_command_rx: broadcast::Receiver<TerminalCommand>,
    term_event_tx: broadcast::Sender<Event>,
    term_event_rx: broadcast::Receiver<Event>,
    term_parser_tx: broadcast::Sender<Event>,
    dom_update_rx: DomUpdateReceiver,
    backend: Arc<B>,
    cancellation_token: CancellationToken,
    parser_running: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    Terminate,
    Suspend,
    Resume,
    Restart,
}

impl<B: Backend + 'static> Runtime<B> {
    pub fn initialize<F, M>(settings: RuntimeSettings, backend: B, f: F) -> Self
    where
        F: FnOnce() -> M + 'static,
        M: Render,
    {
        let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
        let backend = Arc::new(backend);

        let (term_parser_tx, _) = broadcast::channel(32);

        let (term_command_tx, runtime_command_tx) = STATE.with(|s| {
            let s = s.get().unwrap().read();
            let s = s.get(&current_runtime).unwrap();
            (s.term_command_tx.clone(), s.runtime_command_tx.clone())
        });

        if !backend.supports_async_input() {
            wasm_compat::spawn(async move {
                loop {
                    wasm_compat::sleep(Duration::from_millis(20)).await;
                    let _ = term_command_tx.send(TerminalCommand::Poll);
                }
            })
        }

        let dom_update_rx = dom_update_receiver();

        // We need to query this info before reading events
        STATE.with(|s| {
            s.get()
                .unwrap()
                .write()
                .get_mut(&current_runtime)
                .unwrap()
                .supports_keyboard_enhancement = backend.supports_keyboard_enhancement()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if settings.enable_signal_handler {
            use async_signal::{Signal, Signals};
            let runtime_command_tx = runtime_command_tx.clone();
            wasm_compat::spawn(async move {
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
                            let _ = runtime_command_tx.send(RuntimeCommand::Suspend);
                        }
                        Signal::Cont => {
                            let _ = runtime_command_tx.send(RuntimeCommand::Resume);
                        }
                        _ => {
                            let _ = runtime_command_tx.send(RuntimeCommand::Terminate);
                        }
                    }
                }
            });
        }

        mount(f);

        let term_command_tx = STATE.with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_command_tx
                .clone()
        });
        let term_event_tx = STATE.with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_tx
                .clone()
        });

        Self {
            term_command_rx: term_command_tx.subscribe(),
            term_event_rx: term_event_tx.subscribe(),
            term_event_tx,
            term_parser_tx,
            runtime_command_rx: runtime_command_tx.subscribe(),
            backend,
            runtime_command_tx,
            settings,
            dom_update_rx,
            cancellation_token: CancellationToken::new(),
            parser_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn setup_terminal(&mut self) -> io::Result<Terminal<B::TuiBackend>> {
        let mut terminal = self.backend.setup_terminal()?;
        let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
        if let Ok(window_size) = terminal.backend_mut().window_size() {
            STATE.with(|s| {
                s.get()
                    .unwrap()
                    .write()
                    .get_mut(&current_runtime)
                    .unwrap()
                    .pixel_size = Some(Size {
                    width: window_size.pixels.width / window_size.columns_rows.width,
                    height: window_size.pixels.height / window_size.columns_rows.height,
                })
            });
        }
        let backend = self.backend.clone();

        if self.settings.enable_input_reader {
            let backend = backend.clone();

            let term_parser_tx = self.term_parser_tx.clone();
            // Reset cancellation token so the next input reader can start
            self.cancellation_token = CancellationToken::new();
            let cancellation_token = self.cancellation_token.clone();
            if backend.supports_async_input() {
                wasm_compat::spawn(async move {
                    backend.read_input(term_parser_tx, cancellation_token).await;
                });
            }

            self.handle_term_events();
        }
        let show_final_output = self.settings.show_final_output;

        STATE.with(|s| {
            *s.get()
                .unwrap()
                .write()
                .get_mut(&current_runtime)
                .unwrap()
                .restore_terminal
                .lock_mut() = Box::new(move || {
                backend.restore_terminal()?;
                if show_final_output {
                    backend.write_all(b"\n")?;
                }
                Ok(())
            })
        });

        Ok(terminal)
    }

    fn handle_term_events(&self) {
        let signal_tx = self.runtime_command_tx.clone();
        let term_event_tx = self.term_event_tx.clone();

        let mut term_parser_rx = self.term_parser_tx.subscribe();
        let hover_debounce = self.settings.hover_debounce.as_millis();
        if self.parser_running.swap(true, Ordering::SeqCst) {
            return;
        }
        let parser_running = self.parser_running.clone();
        wasm_compat::spawn(async move {
            let mut last_move_time = wasm_compat::now();
            let mut pending_move = None;
            loop {
                let send_next_move = wasm_compat::sleep(Duration::from_millis(
                    hover_debounce.saturating_sub((wasm_compat::now() - last_move_time) as u128)
                        as u64,
                ));

                tokio::select! {
                    next_event = term_parser_rx.recv() => {
                        match next_event {
                            Ok(
                                event @ Event::Key(KeyEvent {
                                    code, modifiers, ..
                                }),
                            ) => {
                                if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
                                    let _ = signal_tx
                                        .send(RuntimeCommand::Terminate)

                                        .tap_err(|_| warn!("error sending terminate signal"));
                                } else if cfg!(unix) && modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('z')
                                {
                                    let _ = signal_tx
                                        .send(RuntimeCommand::Suspend)

                                        .tap_err(|_| warn!("error sending suspend signal"));
                                } else {
                                    let _ = term_event_tx
                                    .send(event)
                                    .tap_err(|_| warn!("error sending terminal event"));
                                }
                            }
                            Ok(
                                mouse_event @ Event::Mouse(MouseEvent {
                                    kind: MouseEventKind::Moved,
                                    ..
                                }),
                            ) => {
                                pending_move = Some(mouse_event);
                                last_move_time = wasm_compat::now();
                            }
                            Ok(event) => {
                                term_event_tx.send(event).ok();

                            }
                            _ => {
                                parser_running.store(false, Ordering::SeqCst);
                                return;
                            }
                        }
                    }
                    _ = send_next_move, if pending_move.is_some() => {
                        term_event_tx.send(pending_move.take().unwrap()).ok();

                    }
                }
            }
        });
    }

    pub async fn run(mut self) -> io::Result<()> {
        let mut terminal = self.setup_terminal()?;
        terminal.draw(|f| render_dom(f.buffer_mut()))?;
        focus_next();

        loop {
            let tick_result = self.tick().await;
            match tick_result {
                TickResult::Redraw => {
                    terminal.draw(|f| render_dom(f.buffer_mut()))?;
                }
                TickResult::Restart => {
                    terminal = self.setup_terminal()?;
                    terminal.draw(|f| render_dom(f.buffer_mut()))?;
                }
                TickResult::Exit => {
                    if !self.settings.show_final_output {
                        terminal.clear()?;
                    }
                    unmount();
                    return Ok(());
                }
                TickResult::Command(command) => {
                    self.handle_terminal_command(command, &mut terminal).await?;
                }
                TickResult::Continue => {}
            }
        }
    }

    pub async fn handle_terminal_command(
        &mut self,
        command: TerminalCommand,
        terminal: &mut Terminal<B::TuiBackend>,
    ) -> io::Result<()> {
        match command {
            TerminalCommand::InsertBefore { height, text } => {
                terminal.insert_before(height, |buf| {
                    Paragraph::new(text).render(buf.area, buf);
                })?;
            }
            TerminalCommand::EnterAltScreen => {
                self.backend.enter_alt_screen(terminal)?;
                terminal.clear()?;
            }
            TerminalCommand::LeaveAltScreen => {
                self.backend.leave_alt_screen(terminal)?;
                terminal.clear()?;
            }
            TerminalCommand::SetTitle(title) => {
                self.backend.set_title(terminal, title)?;
            }
            TerminalCommand::Poll => {
                self.backend.poll_input(terminal, &self.term_parser_tx)?;
            }
            #[cfg(feature = "clipboard")]
            TerminalCommand::SetClipboard(content, kind) => {
                self.backend.set_clipboard(terminal, content, kind)?;
            }
            #[cfg(not(target_arch = "wasm32"))]
            TerminalCommand::Exec { command, on_finish } => {
                self.cancellation_token.cancel();

                restore_terminal()?;
                terminal.clear()?;
                let mut child = command.lock().unwrap().spawn()?;
                let child_stdout = child.stdout.take();
                let child_stderr = child.stderr.take();
                tokio::select! {
                    status = child.wait() => {
                        let status = status.unwrap();
                        let on_finish = (*on_finish.lock().unwrap()).take().unwrap();
                        on_finish(status, child_stdout, child_stderr);
                        self.runtime_command_tx.send(RuntimeCommand::Restart).unwrap();
                    },
                    // Interrupt child if a signal is received
                    res = self.runtime_command_rx.recv() => {
                        child.kill().await.unwrap();
                        if let Ok(signal) = res {
                            self.runtime_command_tx.send(signal).unwrap();
                        }
                    }
                }
            }
        };
        terminal.draw(|f| render_dom(f.buffer_mut()))?;
        Ok(())
    }

    pub async fn tick(&mut self) -> TickResult {
        tokio::select! {
            signal = self.runtime_command_rx.recv() => {
                match signal {
                    Ok(RuntimeCommand::Suspend) => {
                        self.cancellation_token.cancel();
                        restore_terminal().unwrap();
                        #[cfg(unix)]
                        signal_hook::low_level::emulate_default_handler(async_signal::Signal::Tstp as i32).unwrap();
                        TickResult::Continue
                    }
                    Ok(RuntimeCommand::Resume) => {
                        #[cfg(unix)]
                        signal_hook::low_level::emulate_default_handler(async_signal::Signal::Cont as i32).unwrap();
                        TickResult::Restart
                    }
                    Ok(RuntimeCommand::Restart) => {
                        TickResult::Restart
                    }
                    Ok(RuntimeCommand::Terminate) | Err(_) => {
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
            term_command = self.term_command_rx.recv() => {
                if let Ok(term_command) = term_command {
                    TickResult::Command(term_command)
                } else {
                    TickResult::Continue
                }
            }
        }
    }
}

pub fn use_keypress() -> ReadSignal<Option<rooibos_dom::KeyEvent>> {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    let mut term_rx = STATE.with(|s| {
        s.get()
            .unwrap()
            .read()
            .get(&current_runtime)
            .unwrap()
            .term_tx
            .subscribe()
    });
    let (term_signal, set_term_signal) = signal(None);
    wasm_compat::spawn_local(async move {
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
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE.with(|s| {
        s.get()
            .unwrap()
            .read()
            .get(&current_runtime)
            .unwrap()
            .supports_keyboard_enhancement
    })
}

pub fn pixel_size() -> Option<Size> {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE.with(|s| {
        s.get()
            .unwrap()
            .read()
            .get(&current_runtime)
            .unwrap()
            .pixel_size
    })
}

pub fn restore_terminal() -> io::Result<()> {
    STATE.with(|s| {
        let r = s.get().unwrap().read();

        for runtime in r.values() {
            runtime.restore_terminal.lock()()?;
        }
        Ok(())
    })
}

pub fn insert_before(height: u16, text: impl Into<Text<'static>>) {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE
        .with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_command_tx
                .send(TerminalCommand::InsertBefore {
                    height,
                    text: text.into(),
                })
        })
        .unwrap();
}

pub fn enter_alt_screen() {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE
        .with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_command_tx
                .send(TerminalCommand::EnterAltScreen)
        })
        .unwrap();
}

pub fn leave_alt_screen() {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE
        .with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_command_tx
                .send(TerminalCommand::LeaveAltScreen)
        })
        .unwrap();
}

pub fn set_title<T: Display>(title: T) {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE
        .with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_command_tx
                .send(TerminalCommand::SetTitle(title.to_string()))
        })
        .unwrap();
}

#[cfg(feature = "clipboard")]
pub fn set_clipboard<T: Display>(title: T, kind: backend::ClipboardKind) {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE
        .with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_command_tx
                .send(TerminalCommand::SetClipboard(title.to_string(), kind))
        })
        .unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn exec<F>(command: tokio::process::Command, on_finish: F)
where
    F: FnOnce(ExitStatus, Option<tokio::process::ChildStdout>, Option<tokio::process::ChildStderr>)
        + Send
        + Sync
        + 'static,
{
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE
        .with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .term_command_tx
                .send(TerminalCommand::Exec {
                    command: Arc::new(std::sync::Mutex::new(command)),
                    on_finish: Arc::new(std::sync::Mutex::new(Some(Box::new(on_finish)))),
                })
        })
        .unwrap();
}

pub fn exit() {
    let current_runtime = CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0);
    STATE
        .with(|s| {
            s.get()
                .unwrap()
                .read()
                .get(&current_runtime)
                .unwrap()
                .runtime_command_tx
                .send(RuntimeCommand::Terminate)
        })
        .unwrap();
}

pub fn set_panic_hook(owner: Owner) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let original_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = restore_terminal();
            unmount();
            owner.cleanup();
            original_hook(panic_info);
        }));
    }

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

pub fn delay<F>(duration: Duration, f: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_compat::spawn_local(async move {
        wasm_compat::sleep(duration).await;
        f.await;
    });
}
