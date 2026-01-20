use std::any::Any;
use std::error::Error;
use std::fmt::Display;
use std::future::{self, Future};
use std::io;
use std::ops::{Deref, DerefMut};
use std::process::ExitStatus;
use std::sync::Arc;
use std::time::Duration;

use background_service::{BackgroundService, LocalBackgroundService, ServiceContext, TaskId};
use educe::Educe;
use ratatui::Terminal;
use ratatui::backend::Backend as TuiBackend;
use ratatui::text::Text;
use rooibos_reactive::graph::computed::ScopedFuture;
use rooibos_reactive::graph::owner::Owner;
use rooibos_theme::{ColorPalette, SetTheme, TermProfile};
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::task::LocalSet;

use crate::{
    ControlFlow, ExitPayload, OsSignal, RuntimeCommand, RuntimeState, with_all_state, with_state,
};

#[cfg(not(target_arch = "wasm32"))]
pub type OnFinishFn = dyn FnOnce(ExitStatus, Option<tokio::process::ChildStdout>, Option<tokio::process::ChildStderr>)
    + Send
    + Sync;

pub trait AsAnyMut: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

type TerminalFn<B> = dyn FnMut(&mut Terminal<B>) + Send;
pub(crate) struct TerminalFnBoxed<B: TuiBackend>(Box<TerminalFn<B>>);

impl<B> Deref for TerminalFnBoxed<B>
where
    B: TuiBackend,
{
    type Target = Box<TerminalFn<B>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> DerefMut for TerminalFnBoxed<B>
where
    B: TuiBackend,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<B> AsAnyMut for TerminalFnBoxed<B>
where
    B: TuiBackend + Send + 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Clone, Educe)]
#[educe(Debug)]
pub enum TerminalCommand {
    InsertBefore {
        height: u16,
        text: Text<'static>,
    },
    EnterAltScreen,
    LeaveAltScreen,
    SetTitle(String),
    SetViewportWidth(Option<u16>),
    SetViewportHeight(Option<u16>),
    Custom(#[educe(Debug(ignore))] Arc<std::sync::Mutex<Box<dyn AsAnyMut>>>),
    SetClipboard(String, rooibos_terminal::ClipboardKind),
    #[cfg(not(target_arch = "wasm32"))]
    Exec {
        #[educe(Debug(ignore))]
        command: Arc<std::sync::Mutex<tokio::process::Command>>,
        #[educe(Debug(ignore))]
        on_finish: Arc<std::sync::Mutex<Option<Box<OnFinishFn>>>>,
    },
}

pub fn restore_all_terminals() -> io::Result<()> {
    with_all_state(|s| {
        for runtime in s.values() {
            runtime.restore_terminal.lock()()?;
        }
        Ok(())
    })
}

pub fn restore_terminal() -> io::Result<()> {
    with_state(|s| {
        s.restore_terminal.lock()()?;
        Ok(())
    })
}

fn send_command(
    command: TerminalCommand,
) -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    with_state(|s| s.term_command_tx.send(command))?;
    Ok(())
}

pub fn insert_before(
    height: u16,
    text: impl Into<Text<'static>>,
) -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    send_command(TerminalCommand::InsertBefore {
        height,
        text: text.into(),
    })
}

pub fn max_viewport_width(
    max_width: impl Into<Option<u16>>,
) -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    send_command(TerminalCommand::SetViewportWidth(max_width.into()))
}

pub fn max_viewport_height(
    max_height: impl Into<Option<u16>>,
) -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    send_command(TerminalCommand::SetViewportHeight(max_height.into()))
}

pub fn enter_alt_screen() -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    send_command(TerminalCommand::EnterAltScreen)
}

pub fn leave_alt_screen() -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    send_command(TerminalCommand::LeaveAltScreen)
}

pub fn set_title<T: Display>(title: T) -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    send_command(TerminalCommand::SetTitle(title.to_string()))
}

pub fn run_with_terminal<F, B>(f: F) -> Result<(), broadcast::error::SendError<TerminalCommand>>
where
    F: FnMut(&mut Terminal<B>) + Send + 'static,
    B: TuiBackend + Send + 'static,
{
    send_command(TerminalCommand::Custom(Arc::new(std::sync::Mutex::new(
        Box::new(TerminalFnBoxed(Box::new(f))),
    ))))
}

pub fn set_clipboard<T: Display>(
    title: T,
    kind: rooibos_terminal::ClipboardKind,
) -> Result<(), broadcast::error::SendError<TerminalCommand>> {
    send_command(TerminalCommand::SetClipboard(title.to_string(), kind))
}

pub fn spawn_service<S: BackgroundService + Send + 'static>(service: S) -> TaskId {
    with_state(|s| s.context.spawn(ScopedBackgroundService::new(service, s)))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_service_on<S: BackgroundService + Send + 'static>(
    service: S,
    handle: &Handle,
) -> TaskId {
    with_state(|s| {
        s.context
            .spawn_on(ScopedBackgroundService::new(service, s), handle)
    })
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
    with_state(|s| {
        s.context
            .spawn_blocking(ScopedBlockingBackgroundService::new(service, s))
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_blocking_service_on<
    S: background_service::BlockingBackgroundService + Send + 'static,
>(
    service: S,
    handle: &Handle,
) -> TaskId {
    with_state(|s| {
        s.context
            .spawn_blocking_on(ScopedBlockingBackgroundService::new(service, s), handle)
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_thread<S: background_service::BlockingBackgroundService + Send + 'static>(
    service: S,
) -> TaskId {
    with_state(|s| {
        s.context
            .spawn_thread(ScopedBlockingBackgroundService::new(service, s))
    })
}

pub fn before_exit_async<F, Fut>(f: F)
where
    F: Fn(ExitPayload) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ControlFlow> + Send + 'static,
{
    with_state(|s| {
        *s.before_exit.lock_mut() = Box::new(move |payload| {
            let out = f(payload);
            Box::pin(out)
        })
    });
}

pub fn on_os_signal<F>(f: F)
where
    F: Fn(OsSignal) -> ControlFlow + Send + 'static,
{
    with_state(|s| {
        *s.on_os_signal.lock_mut() = Box::new(move |payload| {
            let out = f(payload);
            Box::pin(future::ready(out))
        })
    });
}

pub fn on_os_signal_async<F, Fut>(f: F)
where
    F: Fn(OsSignal) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ControlFlow> + Send + 'static,
{
    with_state(|s| {
        *s.on_os_signal.lock_mut() = Box::new(move |payload| {
            let out = f(payload);
            Box::pin(out)
        })
    });
}

pub fn before_exit<F>(f: F)
where
    F: Fn(ExitPayload) -> ControlFlow + Send + 'static,
{
    with_state(|s| {
        *s.before_exit.lock_mut() = Box::new(move |payload| {
            let out = f(payload);
            Box::pin(future::ready(out))
        })
    });
}

fn exit_with_code_or_error(res: Result<proc_exit::Code, Arc<Box<dyn Error + Send + Sync>>>) {
    with_state(|s| {
        s.runtime_command_tx
            .send(RuntimeCommand::Terminate(res))
            .unwrap()
    });
}

pub fn exit() {
    exit_with_code_or_error(Ok(proc_exit::Code::SUCCESS))
}

pub fn exit_with_code(code: proc_exit::Code) {
    exit_with_code_or_error(Ok(code))
}

pub fn exit_with_error(error: impl Into<Box<dyn Error + Send + Sync>>) {
    exit_with_code_or_error(Err(Arc::new(error.into())))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn exec<F>(command: tokio::process::Command, on_finish: F)
where
    F: FnOnce(ExitStatus, Option<tokio::process::ChildStdout>, Option<tokio::process::ChildStderr>)
        + Send
        + Sync
        + 'static,
{
    with_state(|s| {
        s.term_command_tx
            .send(TerminalCommand::Exec {
                command: Arc::new(std::sync::Mutex::new(command)),
                on_finish: Arc::new(std::sync::Mutex::new(Some(Box::new(on_finish)))),
            })
            .unwrap();
    });
}

struct ScopedBackgroundService<S> {
    service: S,
    owner: Owner,
    profile: TermProfile,
    palette: ColorPalette,
}

impl<S> ScopedBackgroundService<S> {
    fn new(service: S, state: &RuntimeState) -> Self {
        Self {
            service,
            owner: Owner::current().unwrap_or_default(),
            profile: state.profile,
            palette: state.palette,
        }
    }
}

impl<S> BackgroundService for ScopedBackgroundService<S>
where
    S: BackgroundService + Send,
{
    fn name(&self) -> &str {
        self.service.name()
    }

    fn shutdown_timeout(&self) -> Duration {
        self.service.shutdown_timeout()
    }

    async fn run(
        self,
        context: ServiceContext,
    ) -> std::result::Result<(), background_service::error::BoxedError> {
        self.owner.set();
        self.palette.set();
        self.profile.set();
        ScopedFuture::new(self.service.run(context)).await
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct ScopedBlockingBackgroundService<S> {
    service: S,
    owner: Owner,
    profile: TermProfile,
    palette: ColorPalette,
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> ScopedBlockingBackgroundService<S> {
    fn new(service: S, state: &RuntimeState) -> Self {
        Self {
            service,
            owner: Owner::current().unwrap_or_default(),
            profile: state.profile,
            palette: state.palette,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> background_service::BlockingBackgroundService for ScopedBlockingBackgroundService<S>
where
    S: background_service::BlockingBackgroundService,
{
    fn name(&self) -> &str {
        self.service.name()
    }

    fn shutdown_timeout(&self) -> Duration {
        self.service.shutdown_timeout()
    }

    fn run(self, context: ServiceContext) -> Result<(), background_service::error::BoxedError> {
        self.owner.set();
        self.palette.set();
        self.profile.set();
        self.service.run(context)
    }
}
