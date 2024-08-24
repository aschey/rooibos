use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub use background_service::ServiceContext;
use background_service::{Manager, TaskId};
use futures_cancel::FutureExt;
use futures_util::FutureExt as _;
use ratatui::backend::Backend as TuiBackend;
use ratatui::layout::Size;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Terminal;
use rooibos_dom::{
    dom_update_receiver, focus_next, mount, render_dom, send_event, set_pixel_size, unmount,
    DomUpdateReceiver, Event, Render,
};
use tokio::sync::broadcast;
pub use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use crate::backend::Backend;
use crate::debounce::Debouncer;
use crate::error::RuntimeError;
use crate::input_handler::InputHandler;
use crate::{
    restore_terminal, wasm_compat, with_state, with_state_mut, ExitResult, RuntimeSettings,
    TerminalCommand, TerminalFnBoxed,
};

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

#[derive(Clone)]
pub enum TickResult {
    Continue,
    Redraw,
    Restart,
    Command(TerminalCommand),
    Exit,
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
        let term_parser_rx = self.term_parser_tx.subscribe();
        let hover_debouncer = Debouncer::new(self.settings.hover_debounce);
        let resize_debouncer = Debouncer::new(self.settings.resize_debounce);
        let parser_running = self.parser_running.clone();
        let is_quit_event = self.settings.is_quit_event.clone();

        self.service_context.spawn(
            ("input_handler", move |context: ServiceContext| async move {
                let mut input_handler = InputHandler {
                    term_parser_rx,
                    signal_tx,
                    term_event_tx,
                    hover_debouncer,
                    resize_debouncer,
                    context,
                    is_quit_event,
                };
                loop {
                    if !input_handler.handle().await {
                        parser_running.store(false, Ordering::SeqCst);
                        return Ok(());
                    }
                }
            }),
        );
    }

    pub async fn run(mut self) -> Result<(), RuntimeError> {
        let mut terminal = self.setup_terminal().map_err(RuntimeError::SetupFailure)?;
        self.draw(&mut terminal);
        focus_next();

        loop {
            let tick_result = self.tick().await?;
            match tick_result {
                TickResult::Redraw => {
                    self.draw(&mut terminal);
                }
                TickResult::Restart => {
                    terminal = self.setup_terminal().map_err(RuntimeError::SetupFailure)?;
                    self.draw(&mut terminal);
                }
                TickResult::Exit => {
                    if self.should_exit().await {
                        self.handle_exit(&mut terminal).await?;
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

    pub async fn handle_exit(
        mut self,
        terminal: &mut Terminal<B::TuiBackend>,
    ) -> Result<(), RuntimeError> {
        if !self.settings.show_final_output {
            terminal.clear().map_err(RuntimeError::IoFailure)?;
        }

        let services_cancel =
            with_state_mut(|s| s.service_manager.take().expect("manager taken").cancel()).shared();

        loop {
            tokio::select! {
                services_result = services_cancel.clone() => {
                    services_result?;
                    let _ = self
                        .service_manager
                        .cancel()
                        .await
                        .inspect_err(|e| error!("internal services failed: {e:?}"));

                    unmount();
                    return Ok(());
                }
                tick_result = self.tick() => {
                    if let TickResult::Redraw = tick_result? {
                        self.draw(terminal);
                    }
                }
            }
        }
    }

    pub async fn handle_terminal_command(
        &mut self,
        command: TerminalCommand,
        terminal: &mut Terminal<B::TuiBackend>,
    ) -> Result<(), RuntimeError> {
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
                let mut terminal_fn = f.lock().expect("lock poisoned");
                let terminal_fn = terminal_fn
                    .as_any_mut()
                    .downcast_mut::<TerminalFnBoxed<B::TuiBackend>>()
                    .expect("invalid downcast");
                terminal_fn(terminal);
            }
            #[cfg(not(target_arch = "wasm32"))]
            TerminalCommand::Exec { command, on_finish } => {
                self.handle_exec(command, terminal, on_finish).await?;
            }
        };
        self.draw(terminal);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn handle_exec(
        &mut self,
        command: Arc<std::sync::Mutex<tokio::process::Command>>,
        terminal: &mut Terminal<B::TuiBackend>,
        on_finish: Arc<std::sync::Mutex<Option<Box<crate::OnFinishFn>>>>,
    ) -> Result<(), crate::error::ExecError> {
        use crate::error::ExecError;

        if let Some(input_task_id) = self.input_task_id.take() {
            let input_service = self
                .service_context
                .take_service(&input_task_id)
                .expect("input service missing");
            input_service.cancel();
            let _ = input_service
                .wait_for_shutdown()
                .await
                .inspect_err(|e| error!("input reader failed: {e:?}"));
        }

        restore_terminal()?;
        terminal.clear()?;
        let mut child = command
            .lock()
            .expect("lock poisoned")
            .spawn()
            .map_err(ExecError::IoFailure)?;

        let child_stdout = child.stdout.take();
        let child_stderr = child.stderr.take();

        tokio::select! {
            status = child.wait() => {
                let status = status?;
                let on_finish = (*on_finish.lock().expect("lock poisoned")).take().expect("on_finish missing");
                on_finish(status, child_stdout, child_stderr);
                let _ = self
                    .runtime_command_tx
                    .send(RuntimeCommand::Restart)
                    .inspect_err(|e| warn!("failed to send restart: {e:?}"));
            },
            // Interrupt child if a signal is received
            res = self.runtime_command_rx.recv() => {
                child.kill().await.map_err(ExecError::ProcessStopFailure)?;
                if let Ok(signal) = res {
                    let _ = self
                        .runtime_command_tx
                        .send(signal)
                        .inspect_err(|e| error!("failed to send command: {e:?}"));
                }
            }
        }
        Ok(())
    }

    fn draw(&self, terminal: &mut Terminal<B::TuiBackend>) {
        terminal
            .draw(|f| render_dom(f.buffer_mut()))
            .expect("draw failed");
    }

    pub async fn tick(&mut self) -> Result<TickResult, RuntimeError> {
        tokio::select! {
            signal = self.runtime_command_rx.recv() => {
                match signal {
                    Ok(RuntimeCommand::Suspend) => {
                        restore_terminal()?;
                        #[cfg(unix)]
                        signal_hook::low_level::emulate_default_handler(async_signal::Signal::Tstp as i32)
                            .map_err(RuntimeError::SignalHandlerFailure)?;
                        Ok(TickResult::Continue)
                    }
                    Ok(RuntimeCommand::Resume) => {
                        #[cfg(unix)]
                        signal_hook::low_level::emulate_default_handler(async_signal::Signal::Cont as i32)
                            .map_err(RuntimeError::SignalHandlerFailure)?;
                        Ok(TickResult::Restart)
                    }
                    Ok(RuntimeCommand::Restart) => {
                        Ok(TickResult::Restart)
                    }
                    Ok(RuntimeCommand::Terminate) | Err(_) => {
                        Ok(TickResult::Exit)
                    }
                }
            }
            _ = self.dom_update_rx.changed() => {
                Ok(TickResult::Redraw)
            }
            term_event = self.term_event_rx.recv() => {
                if let Ok(term_event) = term_event {
                    send_event(term_event);
                }
                Ok(TickResult::Continue)
            }
            term_command = self.term_command_rx.recv() => {
                if let Ok(term_command) = term_command {
                    Ok(TickResult::Command(term_command))
                } else {
                    Ok(TickResult::Continue)
                }
            }
        }
    }
}

fn spawn_signal_handler(
    runtime_command_tx: &broadcast::Sender<RuntimeCommand>,
    service_context: &ServiceContext,
    settings: &RuntimeSettings,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::signal_handler::SignalHandler;

        let runtime_command_tx = runtime_command_tx.clone();
        let enable_internal_handler = settings.enable_signal_handler;
        service_context.spawn((
            "signal_handler",
            move |context: ServiceContext| async move {
                let signal_handler = SignalHandler {
                    runtime_command_tx,
                    enable_internal_handler,
                    context,
                };
                signal_handler.run().await?;
                Ok(())
            },
        ));
    }
}
