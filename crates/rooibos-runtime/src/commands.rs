use std::fmt::Display;
use std::future::Future;
use std::io;
use std::process::ExitStatus;
use std::sync::Arc;

use background_service::{BackgroundService, LocalBackgroundService, TaskId};
use ratatui::backend::Backend as TuiBackend;
use ratatui::text::Text;
use ratatui::Terminal;
use tokio::runtime::Handle;
use tokio::task::LocalSet;

use crate::{
    backend, with_all_state, with_state, ExitResult, RuntimeCommand, TerminalCommand,
    TerminalFnBoxed,
};

pub fn restore_terminal() -> io::Result<()> {
    with_all_state(|s| {
        for runtime in s.values() {
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
    with_state(|s| {
        s.runtime_command_tx
            .send(RuntimeCommand::Terminate)
            .unwrap()
    });
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
