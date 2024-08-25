use std::io;
use std::process::ExitStatus;

pub use background_service::error::BackgroundServiceErrors;

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("terminal setup failed: {0}")]
    SetupFailure(io::Error),
    #[error("services failed: {0}")]
    ServicesFailure(#[from] BackgroundServiceErrors),
    #[error("process exec failed: {0}")]
    ExecFailure(#[from] ExecError),
    #[error("signal handler failed: {0}")]
    SignalHandlerFailure(io::Error),
    #[error("I/O failure: {0}")]
    IoFailure(#[from] io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ExecError {
    #[error("child process failed: {0}")]
    ChildProcessFailure(ExitStatus),
    #[error("child process failed to spawn: {0}")]
    ProcessSpawnFailure(io::Error),
    #[error("child process failed to stop: {0}")]
    ProcessStopFailure(io::Error),
    #[error("I/O failure: {0}")]
    IoFailure(#[from] io::Error),
}