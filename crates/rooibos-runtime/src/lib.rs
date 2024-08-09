pub mod backend;

use std::any::Any;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::Display;
use std::future::Future;
use std::io;
use std::panic::{set_hook, take_hook};
use std::pin::Pin;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use backend::Backend;
pub use background_service::ServiceContext;
use background_service::{BackgroundService, LocalBackgroundService, Manager, TaskId};
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
use tokio::runtime::Handle;
use tokio::sync::{broadcast, mpsc};
use tokio::task::LocalSet;
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

type ExitResultFuture = dyn Future<Output = ExitResult> + Send;

struct RuntimeState {
    term_tx: broadcast::Sender<rooibos_dom::Event>,
    term_command_tx: broadcast::Sender<TerminalCommand>,
    runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    supports_keyboard_enhancement: bool,
    pixel_size: Option<Size>,
    service_manager: Option<Manager>,
    context: ServiceContext,
    restore_terminal: wasm_compat::Mutex<Box<RestoreFn>>,
    before_exit: wasm_compat::Mutex<Box<dyn Fn() -> Pin<Box<ExitResultFuture>> + Send>>,
}

impl RuntimeState {
    fn new() -> Self {
        let (term_tx, _) = broadcast::channel(32);
        let (term_command_tx, _) = broadcast::channel(32);
        let (runtime_command_tx, _) = broadcast::channel(32);
        let cancellation_token = CancellationToken::new();
        let service_manager = Manager::new(
            cancellation_token.clone(),
            background_service::Settings::default(),
        );
        Self {
            term_tx,
            term_command_tx,
            runtime_command_tx,
            supports_keyboard_enhancement: false,
            pixel_size: None,
            restore_terminal: wasm_compat::Mutex::new(Box::new(|| Ok(()))),
            before_exit: wasm_compat::Mutex::new(Box::new(move || {
                Box::pin(async move { ExitResult::Exit })
            })),
            context: service_manager.get_context(),
            service_manager: Some(service_manager),
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

pub trait AsAnyMut: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

type TerminalFn<B> = dyn FnMut(&mut Terminal<B>) + Send;

struct TerminalFnBoxed<B: TuiBackend>(Box<TerminalFn<B>>);

impl<B> AsAnyMut for TerminalFnBoxed<B>
where
    B: TuiBackend + Send + 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

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
    Custom(#[derivative(Debug = "ignore")] Arc<std::sync::Mutex<Box<dyn AsAnyMut>>>),
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
    parser_running: Arc<AtomicBool>,
    input_task_id: Option<TaskId>,
    service_manager: Manager,
    service_context: ServiceContext,
}

#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    Terminate,
    Suspend,
    Resume,
    Restart,
}

impl<B: Backend + 'static> Runtime<B> {
    pub fn initialize<F, M>(backend: B, f: F) -> Self
    where
        F: FnOnce() -> M + 'static,
        M: Render,
        <M as Render>::DomState: 'static,
    {
        Self::initialize_with_settings(RuntimeSettings::default(), backend, f)
    }

    pub fn initialize_with_settings<F, M>(settings: RuntimeSettings, backend: B, f: F) -> Self
    where
        F: FnOnce() -> M + 'static,
        M: Render,
        <M as Render>::DomState: 'static,
    {
        let backend = Arc::new(backend);

        let (term_parser_tx, _) = broadcast::channel(32);

        let (term_command_tx, runtime_command_tx) =
            with_state(|s| (s.term_command_tx.clone(), s.runtime_command_tx.clone()));
        let service_manager = Manager::new(
            CancellationToken::new(),
            background_service::Settings::default(),
        );
        let service_context = service_manager.get_context();

        if !backend.supports_async_input() {
            service_context.spawn(("input_poller", |context: ServiceContext| async move {
                loop {
                    tokio::select! {
                        _ =  wasm_compat::sleep(Duration::from_millis(20)) => {
                            let _ = term_command_tx.send(TerminalCommand::Poll);
                        }
                        _ = context.cancelled() => {
                            return Ok(());
                        }
                    }
                }
            }));
        }

        let dom_update_rx = dom_update_receiver();

        // We need to query this info before reading events
        with_state_mut(|s| {
            s.supports_keyboard_enhancement = backend.supports_keyboard_enhancement()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if settings.enable_signal_handler {
            use async_signal::{Signal, Signals};
            let runtime_command_tx = runtime_command_tx.clone();
            service_context.spawn(("signal_handler", |context: ServiceContext| async move {
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

                loop {
                    tokio::select! {
                        Some(Ok(signal)) = signals.next() => {
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
                        _ = context.cancelled() => {
                            return Ok(());
                        }
                    }
                }
            }));
        }

        mount(f);

        let term_command_tx = with_state(|s| s.term_command_tx.clone());
        let term_event_tx = with_state(|s| s.term_tx.clone());

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
            parser_running: Arc::new(AtomicBool::new(false)),
            input_task_id: None,
            service_manager,
            service_context,
        }
    }

    pub fn setup_terminal(&mut self) -> io::Result<Terminal<B::TuiBackend>> {
        let mut terminal = self.backend.setup_terminal()?;

        if let Ok(window_size) = terminal.backend_mut().window_size() {
            with_state_mut(|s| {
                s.pixel_size = Some(Size {
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
            // self.cancellation_token = CancellationToken::new();
            // let cancellation_token = self.cancellation_token.clone();
            if backend.supports_async_input() {
                self.input_task_id = Some(self.service_context.spawn((
                    "input_reader",
                    move |context: ServiceContext| async move {
                        backend.read_input(term_parser_tx, context).await;
                        Ok(())
                    },
                )));
            }

            self.handle_term_events();
        }
        let show_final_output = self.settings.show_final_output;

        with_state(|s| {
            *s.restore_terminal.lock_mut() = Box::new(move || {
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

        self.service_context.spawn(("input_handler",move |context: ServiceContext| async move {
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
                            Err(_) => {
                                parser_running.store(false, Ordering::SeqCst);
                                return Ok(());
                            }
                        }
                    }
                    _ = context.cancelled() => {
                        parser_running.store(false, Ordering::SeqCst);
                        return Ok(());
                    }
                    _ = send_next_move, if pending_move.is_some() => {
                        term_event_tx.send(pending_move.take().unwrap()).ok();
                    }
                }
            }
        }));
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
                    if self.should_exit().await {
                        self.handle_exit(&mut terminal).await.unwrap();
                        return Ok(());
                    }
                }
                TickResult::Command(command) => {
                    self.handle_terminal_command(command, &mut terminal).await?;
                }
                TickResult::Continue => {}
            }
        }
    }

    pub async fn should_exit(&self) -> bool {
        let _guard = reactive_graph::diagnostics::SpecialNonReactiveZone::enter();
        let exit_result = with_state_mut(|s| (s.before_exit.lock())());
        exit_result.await == ExitResult::Exit
    }

    pub async fn handle_exit(mut self, terminal: &mut Terminal<B::TuiBackend>) -> io::Result<()> {
        if !self.settings.show_final_output {
            terminal.clear()?;
        }

        let cancel_fut = with_state_mut(|s| s.service_manager.take().unwrap().cancel());
        let (tx, mut rx) = mpsc::channel(1);
        wasm_compat::spawn(async move {
            let res = cancel_fut.await;
            tx.send(res).await.unwrap();
        });
        loop {
            tokio::select! {
                res = rx.recv() => {
                    res.unwrap().unwrap();
                    self.service_manager.cancel().await.unwrap();
                    unmount();
                    return Ok(());
                }
                tick_result = self.tick() => {
                    match tick_result {
                        TickResult::Redraw => {
                            terminal.draw(|f| render_dom(f.buffer_mut()))?;
                        }
                        TickResult::Continue => {}
                        _ => {}
                    }
                }
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
            TerminalCommand::Custom(f) => {
                let mut terminal_fn = f.lock().unwrap();

                let terminal_fn = terminal_fn
                    .as_any_mut()
                    .downcast_mut::<TerminalFnBoxed<B::TuiBackend>>()
                    .unwrap();
                terminal_fn.0(terminal);
            }
            #[cfg(not(target_arch = "wasm32"))]
            TerminalCommand::Exec { command, on_finish } => {
                if let Some(input_task_id) = self.input_task_id.take() {
                    let input_service = self.service_context.take_service(&input_task_id).unwrap();
                    input_service.cancel();
                    input_service.wait_for_shutdown().await.unwrap();
                }

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
                        // self.cancellation_token.cancel();
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
    let mut term_rx = with_state(|s| s.term_tx.subscribe());
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
    with_state(|s| s.supports_keyboard_enhancement)
}

pub fn pixel_size() -> Option<Size> {
    with_state(|s| s.pixel_size)
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
    with_state(|s| {
        s.term_command_tx.send(TerminalCommand::InsertBefore {
            height,
            text: text.into(),
        })
    })
    .unwrap();
}

pub fn enter_alt_screen() {
    with_state(|s| s.term_command_tx.send(TerminalCommand::EnterAltScreen)).unwrap();
}

pub fn leave_alt_screen() {
    with_state(|s| s.term_command_tx.send(TerminalCommand::LeaveAltScreen)).unwrap();
}

pub fn set_title<T: Display>(title: T) {
    with_state(|s| {
        s.term_command_tx
            .send(TerminalCommand::SetTitle(title.to_string()))
    })
    .unwrap();
}

pub fn run_with_terminal<F, B>(f: F)
where
    F: FnMut(&mut Terminal<B>) + Send + 'static,
    B: TuiBackend + Send + 'static,
{
    with_state(|s| {
        s.term_command_tx
            .send(TerminalCommand::Custom(Arc::new(std::sync::Mutex::new(
                Box::new(TerminalFnBoxed(Box::new(f))),
            ))))
    })
    .unwrap();
}

pub fn spawn_service<S: BackgroundService + Send + 'static>(service: S) -> TaskId {
    with_state(|s| s.context.spawn(service))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_service_on<S: BackgroundService + Send + 'static>(
    service: S,
    handle: &Handle,
) -> TaskId {
    with_state(|s| s.context.spawn_on(service, handle))
}

pub fn spawn_local_service<S: LocalBackgroundService + 'static>(service: S) -> TaskId {
    with_state(|s| s.context.spawn_local(service))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_local_service_on<S: LocalBackgroundService + 'static>(
    service: S,
    local_set: &LocalSet,
) -> TaskId {
    with_state(|s| s.context.spawn_local_on(service, local_set))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_blocking_service<S: background_service::BlockingBackgroundService + Send + 'static>(
    service: S,
) -> TaskId {
    with_state(|s| s.context.spawn_blocking(service))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_blocking_service_on<
    S: background_service::BlockingBackgroundService + Send + 'static,
>(
    service: S,
    handle: &Handle,
) -> TaskId {
    with_state(|s| s.context.spawn_blocking_on(service, handle))
}

#[cfg(feature = "clipboard")]
pub fn set_clipboard<T: Display>(title: T, kind: backend::ClipboardKind) {
    with_state(|s| {
        s.term_command_tx
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExitResult {
    Exit,
    PreventExit,
}

pub fn before_exit<F, Fut>(f: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ExitResult> + Send + 'static,
{
    with_state(|s| {
        *s.before_exit.lock_mut() = Box::new(move || {
            let out = f();
            Box::pin(out)
        })
    });
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

fn current_runtime() -> u32 {
    CURRENT_RUNTIME.try_with(|c| *c).unwrap_or(0)
}

fn with_state<F: FnOnce(&RuntimeState) -> T, T>(f: F) -> T {
    STATE.with(|s| f(s.get().unwrap().read().get(&current_runtime()).unwrap()))
}

fn with_state_mut<F: FnOnce(&mut RuntimeState) -> T, T>(f: F) -> T {
    STATE.with(|s| {
        f(s.get()
            .unwrap()
            .write()
            .get_mut(&current_runtime())
            .unwrap())
    })
}
