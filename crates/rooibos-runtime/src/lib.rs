use std::future::Future;
use std::io;
use std::panic::{set_hook, take_hook};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use backend::Backend;
pub use background_service::ServiceContext;
use background_service::{Manager, TaskId};
pub use commands::*;
use debounce::Debouncer;
use futures_cancel::FutureExt;
use futures_util::{FutureExt as _, StreamExt};
use ratatui::backend::Backend as TuiBackend;
use ratatui::layout::Size;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Terminal;
use reactive_graph::owner::Owner;
use reactive_graph::signal::{signal, ReadSignal};
use reactive_graph::traits::{IsDisposed, Set};
use rooibos_dom::{
    dom_update_receiver, focus_next, mount, render_dom, send_event, set_pixel_size, unmount,
    DomUpdateReceiver, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, Render,
};
pub use state::*;
use tap::TapFallible;
use tokio::sync::broadcast;
use tokio::task;
pub use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

pub mod backend;
mod commands;
mod debounce;
mod state;

pub mod wasm_compat {
    pub use ::wasm_compat::cell::*;
    pub use ::wasm_compat::futures::*;
    pub use ::wasm_compat::once::*;
    pub use ::wasm_compat::static_init;
    pub use ::wasm_compat::static_init::*;
    pub use ::wasm_compat::sync::*;
    pub use ::wasm_compat::time::*;
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
    let owner = Owner::new();
    set_panic_hook(owner.clone());
    let res = owner.with(f);

    owner.cleanup();
    drop(owner);

    let _ = restore_terminal().tap_err(|e| error!("error restoring terminal: {e:?}"));
    res
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn run_with_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    init_executor();
    let local = task::LocalSet::new();
    local.run_until(f).await
}

#[cfg(target_arch = "wasm32")]
pub async fn run_with_executor<T, F>(f: F) -> T
where
    F: Future<Output = T>,
{
    init_executor();
    f.await
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_executor() {
    any_spawner::Executor::init_tokio().expect("executor already initialized");
}

#[cfg(target_arch = "wasm32")]
pub fn init_executor() {
    any_spawner::Executor::init_wasm_bindgen().expect("executor already initialized");
}

#[derive(Debug)]
pub struct RuntimeSettings {
    enable_input_reader: bool,
    enable_signal_handler: bool,
    show_final_output: bool,
    hover_debounce: Duration,
    resize_debounce: Duration,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            enable_input_reader: true,
            enable_signal_handler: true,
            show_final_output: true,
            hover_debounce: Duration::from_millis(20),
            resize_debounce: Duration::from_millis(20),
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

    pub fn resize_debounce(mut self, resize_debounce: Duration) -> Self {
        self.resize_debounce = resize_debounce;
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
                    if wasm_compat::sleep(Duration::from_millis(20))
                        .cancel_with(context.cancelled())
                        .await
                        .is_ok()
                    {
                        let _ = term_command_tx.send(TerminalCommand::Poll);
                    } else {
                        return Ok(());
                    }
                }
            }));
        }

        let dom_update_rx = dom_update_receiver();
        // We need to query this info before reading events
        let _ =
            rooibos_dom::set_supports_keyboard_enhancement(backend.supports_keyboard_enhancement());
        spawn_signal_handler(&runtime_command_tx, &service_context, &settings);

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

        let window_size = terminal.backend_mut().window_size().ok();
        let _ = set_pixel_size(window_size.map(|s| Size {
            width: s.pixels.width / s.columns_rows.width,
            height: s.pixels.height / s.columns_rows.height,
        }));

        let backend = self.backend.clone();
        if self.settings.enable_input_reader {
            let backend = backend.clone();
            let term_parser_tx = self.term_parser_tx.clone();

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
        if self.parser_running.swap(true, Ordering::SeqCst) {
            return;
        }
        let signal_tx = self.runtime_command_tx.clone();
        let term_event_tx = self.term_event_tx.clone();
        let mut term_parser_rx = self.term_parser_tx.subscribe();
        let mut hover_debouncer = Debouncer::new(self.settings.hover_debounce);
        let mut resize_debouncer = Debouncer::new(self.settings.resize_debounce);
        let parser_running = self.parser_running.clone();

        self.service_context.spawn(
            ("input_handler", move |context: ServiceContext| async move {
                loop {
                    tokio::select! {
                        next_event = term_parser_rx.recv() => {
                            if !handle_term_event(
                                next_event,
                                &signal_tx,
                                &term_event_tx,
                                &mut hover_debouncer,
                                &mut resize_debouncer,
                            ).await {
                                parser_running.store(false, Ordering::SeqCst);
                                return Ok(());
                            }
                        }
                        _ = context.cancelled() => {
                            parser_running.store(false, Ordering::SeqCst);
                            return Ok(());
                        }
                        pending_move = hover_debouncer.next_value() => {
                            term_event_tx.send(pending_move).ok();
                        }
                        pending_resize = resize_debouncer.next_value() => {
                            term_event_tx.send(pending_resize).ok();
                        }
                    }
                }
            }),
        );
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

        let cancel_fut = with_state_mut(|s| s.service_manager.take().unwrap().cancel()).shared();
        loop {
            tokio::select! {
                res = cancel_fut.clone() => {
                    res.unwrap();
                    self.service_manager.cancel().await.unwrap();
                    unmount();
                    return Ok(());
                }
                tick_result = self.tick() => {
                    if let TickResult::Redraw = tick_result {
                        terminal.draw(|f| render_dom(f.buffer_mut()))?;
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
                terminal_fn(terminal);
            }
            #[cfg(not(target_arch = "wasm32"))]
            TerminalCommand::Exec { command, on_finish } => {
                self.handle_exec(command, terminal, on_finish).await?;
            }
        };
        terminal.draw(|f| render_dom(f.buffer_mut()))?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn handle_exec(
        &mut self,
        command: Arc<std::sync::Mutex<tokio::process::Command>>,
        terminal: &mut Terminal<B::TuiBackend>,
        on_finish: Arc<std::sync::Mutex<Option<Box<OnFinishFn>>>>,
    ) -> io::Result<()> {
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
        Ok(())
    }

    pub async fn tick(&mut self) -> TickResult {
        tokio::select! {
            signal = self.runtime_command_rx.recv() => {
                match signal {
                    Ok(RuntimeCommand::Suspend) => {
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

async fn handle_term_event(
    next_event: Result<Event, broadcast::error::RecvError>,
    signal_tx: &broadcast::Sender<RuntimeCommand>,
    term_event_tx: &broadcast::Sender<Event>,
    hover_debouncer: &mut Debouncer<Event>,
    resize_debouncer: &mut Debouncer<Event>,
) -> bool {
    match next_event {
        Ok(
            event @ Event::Key(KeyEvent {
                code, modifiers, ..
            }),
        ) => {
            handle_key_event(event, code, modifiers, signal_tx, term_event_tx);
        }
        Ok(
            mouse_event @ Event::Mouse(MouseEvent {
                kind: MouseEventKind::Moved,
                ..
            }),
        ) => {
            hover_debouncer.update(mouse_event).await;
        }
        Ok(resize_event @ Event::Resize(_, _)) => {
            resize_debouncer.update(resize_event).await;
        }
        Ok(event) => {
            term_event_tx.send(event).ok();
        }
        Err(_) => {
            return false;
        }
    }
    true
}

fn handle_key_event(
    event: Event,
    code: KeyCode,
    modifiers: KeyModifiers,
    signal_tx: &broadcast::Sender<RuntimeCommand>,
    term_event_tx: &broadcast::Sender<Event>,
) {
    if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
        let _ = signal_tx
            .send(RuntimeCommand::Terminate)
            .tap_err(|_| warn!("error sending terminate signal"));
    } else if cfg!(unix) && modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('z') {
        // Defer to the external stream for suspend commands if it exists
        if !has_external_signal_stream() {
            let _ = signal_tx
                .send(RuntimeCommand::Suspend)
                .tap_err(|_| warn!("error sending suspend signal"));
        }
    } else {
        let _ = term_event_tx
            .send(event)
            .tap_err(|_| warn!("error sending terminal event"));
    }
}

#[cfg(target_arch = "wasm32")]
fn spawn_signal_handler(
    runtime_command_tx: &broadcast::Sender<RuntimeCommand>,
    service_context: &ServiceContext,
    settings: &RuntimeSettings,
) {
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_signal_handler(
    runtime_command_tx: &broadcast::Sender<RuntimeCommand>,
    service_context: &ServiceContext,
    settings: &RuntimeSettings,
) {
    use async_signal::{Signal, Signals};

    let runtime_command_tx = runtime_command_tx.clone();
    if let Some(mut signals) = get_external_signal_stream() {
        service_context.spawn(("signal_handler", |context: ServiceContext| async move {
            while let Ok(Ok(signal)) = signals.recv().cancel_with(context.cancelled()).await {
                handle_signal(&runtime_command_tx, signal);
            }
            Ok(())
        }));
    } else if settings.enable_signal_handler {
        service_context.spawn(("signal_handler", |context: ServiceContext| async move {
            #[cfg(unix)]
            // SIGSTP cannot be handled
            // https://www.gnu.org/software/libc/manual/html_node/Job-Control-Signals.html
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

            while let Ok(Some(Ok(signal))) = signals.next().cancel_with(context.cancelled()).await {
                handle_signal(&runtime_command_tx, signal);
            }
            Ok(())
        }));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_signal(
    runtime_command_tx: &broadcast::Sender<RuntimeCommand>,
    signal: async_signal::Signal,
) {
    use async_signal::Signal;
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

pub fn use_keypress() -> ReadSignal<Option<rooibos_dom::KeyEvent>> {
    let mut term_rx = with_state(|s| s.term_tx.subscribe());
    let (term_signal, set_term_signal) = signal(None);
    wasm_compat::spawn_local(async move {
        while let Ok(event) = term_rx.recv().await {
            if term_signal.is_disposed() {
                return;
            }
            if let Event::Key(key_event) = event {
                if key_event.kind == rooibos_dom::KeyEventKind::Press {
                    set_term_signal.set(Some(key_event));
                }
            }
        }
    });
    term_signal
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
