use std::error::Error;
use std::io;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub use background_service::ServiceContext;
use background_service::{Manager, TaskId};
use futures_cancel::FutureExt;
use futures_util::{FutureExt as _, StreamExt, pin_mut};
use ratatui::Viewport;
use ratatui::layout::Position;
use rooibos_dom::events::dispatch_event;
use rooibos_dom::{
    DomUpdateReceiver, Event, NonblockingTerminal, dom_update_receiver, focus_next,
    render_terminal, unmount,
};
use rooibos_reactive::dom::{Render, mount};
use rooibos_terminal::{self, Backend};
use tokio::sync::broadcast;
pub use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use crate::debounce::Debouncer;
use crate::error::RuntimeError;
use crate::input_handler::InputHandler;
use crate::signal_handler::signal;
use crate::{
    ExitPayload, ExitResult, RuntimeSettings, TerminalCommand, TerminalFnBoxed, restore_terminal,
    set_panic_hook, wasm_compat, with_state, with_state_mut,
};

#[derive(Debug)]
pub struct Runtime<B: Backend> {
    settings: RuntimeSettings,
    runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    runtime_command_rx: broadcast::Receiver<RuntimeCommand>,
    render_debouncer: Debouncer<()>,
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
    Terminate(Result<proc_exit::Code, Arc<Box<dyn Error + Send + Sync>>>),
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
    Exit(ExitPayload),
}

impl<B> Runtime<B>
where
    B: Backend + 'static,
    B::TuiBackend: wasm_compat::Send + wasm_compat::Sync + 'static,
{
    pub fn initialize(backend: B) -> Self {
        Self::initialize_with(RuntimeSettings::default(), backend)
    }

    pub fn initialize_with(settings: RuntimeSettings, backend: B) -> Self {
        set_panic_hook();
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

        let term_command_tx = with_state(|s| s.term_command_tx.clone());
        let term_event_tx = with_state(|s| s.term_tx.clone());
        const MILLIS_PER_SEC: f32 = 1000.0;
        Self {
            term_command_rx: term_command_tx.subscribe(),
            term_event_rx: term_event_tx.subscribe(),
            term_event_tx,
            term_parser_tx,
            render_debouncer: Debouncer::new(Duration::from_millis(
                (MILLIS_PER_SEC / settings.max_fps) as u64,
            )),
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

    pub fn mount<F, M>(&self, f: F)
    where
        F: FnOnce() -> M + 'static,
        M: Render,
        <M as Render>::DomState: 'static,
    {
        let window_size = self.backend.window_size().ok();
        mount(f, window_size);
    }

    pub async fn setup_terminal(&mut self) -> io::Result<NonblockingTerminal<B::TuiBackend>> {
        let tui_backend = self.backend.create_tui_backend()?;
        let mut terminal = ratatui::Terminal::with_options(
            tui_backend,
            ratatui::TerminalOptions {
                viewport: self.settings.viewport.clone(),
            },
        )?;
        self.backend.setup_terminal(terminal.backend_mut())?;
        let terminal = NonblockingTerminal::new(terminal);

        if self.settings.enable_input_reader {
            let term_parser_tx = self.term_parser_tx.clone();

            if self.backend.supports_async_input() {
                let input_stream = self.backend.async_input_stream();
                self.input_task_id = Some(self.service_context.spawn((
                    "input_reader",
                    move |context: ServiceContext| async move {
                        pin_mut!(input_stream);

                        while let Ok(Some(event)) =
                            input_stream.next().cancel_with(context.cancelled()).await
                        {
                            let _ = term_parser_tx.send(event);
                        }
                        Ok(())
                    },
                )));
            }
            self.handle_term_events();
        }
        let show_final_output = self.show_final_output();

        let backend = self.backend.clone();
        with_state(|s| {
            *s.restore_terminal.lock_mut() = Box::new(move || {
                backend.restore_terminal()?;
                if show_final_output {
                    // ensure we start a new line before exiting to ensure the full content is shown
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
        let event_filter = self.settings.event_filter.clone();
        // Make sure we load the editing variable before spawning, because it's a thread local
        let editing = rooibos_dom::editing();
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
                    editing,
                    event_filter,
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

    fn show_final_output(&self) -> bool {
        // Default to true on fixed and inline viewports, false on fullscreen unless explicitly
        // requested otherwise
        self.settings
            .show_final_output
            .unwrap_or(match self.settings.viewport {
                Viewport::Fixed(_) | Viewport::Inline(_) => true,
                Viewport::Fullscreen => false,
            })
    }

    pub async fn run<F, M>(mut self, f: F) -> Result<ExitCode, RuntimeError>
    where
        F: FnOnce() -> M + 'static,
        M: Render,
        <M as Render>::DomState: 'static,
    {
        self.mount(f);
        let mut terminal = self
            .setup_terminal()
            .await
            .map_err(RuntimeError::SetupFailure)?;
        self.draw(&mut terminal).await;
        focus_next();

        loop {
            let tick_result = self.tick().await?;
            match tick_result {
                TickResult::Redraw => {
                    self.draw(&mut terminal).await;
                }
                TickResult::Restart => {
                    terminal.join().await;
                    terminal = self
                        .setup_terminal()
                        .await
                        .map_err(RuntimeError::SetupFailure)?;
                    self.draw(&mut terminal).await;
                }
                TickResult::Exit(payload) => {
                    // Redraw one last time in case the screen was updated after the last debounce
                    // tick
                    self.draw(&mut terminal).await;
                    if self.should_exit(payload.clone()).await {
                        self.handle_exit(&mut terminal).await?;
                        restore_terminal()?;
                        if let Some(e) = payload.error() {
                            return Err(RuntimeError::UserDefined(e.clone()));
                        } else {
                            return Ok(payload.code().as_exit_code().unwrap_or(ExitCode::FAILURE));
                        }
                    }
                }
                TickResult::Command(command) => {
                    self.handle_terminal_command(command, &mut terminal).await?;
                }
                TickResult::Continue => {}
            }
        }
    }

    pub async fn should_exit(&self, payload: ExitPayload) -> bool {
        let exit_result = with_state_mut(|s| (s.before_exit.lock())(payload));
        exit_result.await == ExitResult::Exit
    }

    pub async fn handle_exit(
        mut self,
        terminal: &mut NonblockingTerminal<B::TuiBackend>,
    ) -> Result<(), RuntimeError> {
        if self.show_final_output() {
            let y = terminal.area().y;
            let height = match self.settings.viewport {
                Viewport::Fullscreen => terminal.size().await?.height,
                Viewport::Inline(size) => size,
                Viewport::Fixed(rect) => rect.height,
            };
            // Move the cursor so we can add a new line at the very bottom
            terminal
                .set_cursor_position(Position {
                    x: 0,
                    y: y + height,
                })
                .await;
        } else {
            terminal.clear().await;
        }

        let services_cancel =
            with_state_mut(|s| s.service_manager.take().expect("manager taken").cancel()).shared();

        loop {
            tokio::select! {
                services_result = services_cancel.clone() => {
                    let _ = services_result.inspect_err(|e| error!("services failed: {e:?}"));
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
                        self.draw(terminal).await;
                    }
                }
            }
        }
    }

    pub async fn handle_terminal_command(
        &mut self,
        command: TerminalCommand,
        terminal: &mut NonblockingTerminal<B::TuiBackend>,
    ) -> Result<(), RuntimeError> {
        match command {
            TerminalCommand::InsertBefore { height, text } => {
                terminal.insert_before(height, text).await;
            }
            TerminalCommand::EnterAltScreen => {
                terminal.with_terminal_mut(|t| self.backend.enter_alt_screen(t.backend_mut()))?;
                terminal.clear().await;
            }
            TerminalCommand::LeaveAltScreen => {
                terminal.with_terminal_mut(|t| self.backend.leave_alt_screen(t.backend_mut()))?;
                terminal.clear().await;
            }
            TerminalCommand::SetTitle(title) => {
                terminal.with_terminal_mut(|t| {
                    self.backend.set_title(t.backend_mut(), title.clone())
                })?;
            }
            TerminalCommand::Poll => {
                terminal.with_terminal_mut(|t| {
                    self.backend
                        .poll_input(t.backend_mut(), &self.term_parser_tx)
                })?;
            }
            #[cfg(feature = "clipboard")]
            TerminalCommand::SetClipboard(content, kind) => {
                terminal.with_terminal_mut(|t| {
                    self.backend
                        .set_clipboard(t.backend_mut(), content.clone(), kind)
                })?;
            }
            TerminalCommand::SetViewportWidth(max_width) => {
                rooibos_dom::max_viewport_width(max_width);
            }
            TerminalCommand::SetViewportHeight(max_height) => {
                rooibos_dom::max_viewport_height(max_height);
            }
            TerminalCommand::Custom(f) => {
                let mut terminal_fn = f.lock().expect("lock poisoned");
                let terminal_fn = terminal_fn
                    .as_any_mut()
                    .downcast_mut::<TerminalFnBoxed<B::TuiBackend>>()
                    .expect("invalid downcast");
                terminal.with_terminal_mut(|t| terminal_fn(t));
            }
            #[cfg(not(target_arch = "wasm32"))]
            TerminalCommand::Exec { command, on_finish } => {
                self.handle_exec(command, terminal, on_finish).await?;
            }
        };
        self.draw(terminal).await;
        Ok(())
    }

    async fn cancel_input_reader(&mut self) {
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
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn handle_exec(
        &mut self,
        command: Arc<std::sync::Mutex<tokio::process::Command>>,
        terminal: &mut NonblockingTerminal<B::TuiBackend>,
        on_finish: Arc<std::sync::Mutex<Option<Box<crate::OnFinishFn>>>>,
    ) -> Result<(), crate::error::ExecError> {
        use crate::error::ExecError;
        self.cancel_input_reader().await;

        restore_terminal()?;
        // enter alt screen to prevent flickering before the new command is shown
        terminal.with_terminal_mut(|t| self.backend.enter_alt_screen(t.backend_mut()))?;

        let mut child = command.lock().expect("lock poisoned").spawn()?;

        let child_stdout = child.stdout.take();
        let child_stderr = child.stderr.take();

        tokio::select! {
            status = child.wait() => {
                // prevent flickering
                terminal.with_terminal_mut(|t| self.backend.enter_alt_screen(t.backend_mut()))?;
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

    pub async fn draw(&self, terminal: &mut NonblockingTerminal<B::TuiBackend>) {
        render_terminal(terminal).await.expect("draw failed");
    }

    pub async fn tick(&mut self) -> Result<TickResult, RuntimeError> {
        tokio::select! {
            command = self.runtime_command_rx.recv() => {
                match command {
                    Ok(RuntimeCommand::Suspend) => {
                        self.cancel_input_reader().await;
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
                    Ok(RuntimeCommand::Terminate(res))  => {
                        Ok(TickResult::Exit(ExitPayload::from_result(res)))
                    }
                    Err(e) => {
                        error!("error receiving runtime command: {e:?}");
                        Ok(TickResult::Exit(ExitPayload::from_result(Ok(signal::Code::FAILURE))))
                    }
                }
            }
            _ = self.dom_update_rx.changed() => {
                self.render_debouncer
                    .update(())
                    .await
                    .map_err(|e| RuntimeError::Internal(e.into()))?;
                Ok(TickResult::Continue)
            }
            Some(()) = self.render_debouncer.next_value() => {
                Ok(TickResult::Redraw)
            }
            term_event = self.term_event_rx.recv() => {
                if let Ok(term_event) = term_event {
                    dispatch_event(term_event.into());
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
