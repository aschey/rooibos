use std::io;

#[cfg(not(target_arch = "wasm32"))]
use async_signal::{Signal, Signals};
use background_service::ServiceContext;
use futures_util::StreamExt;
use tokio::sync::broadcast;
#[cfg(not(target_arch = "wasm32"))]
use tokio_util::future::FutureExt;
use tracing::error;

use crate::RuntimeCommand;

pub mod proc_exit {
    pub use proc_exit::Code;
    pub use proc_exit::bash::{SIGABRT, SIGINT, SIGQUIT, SIGTERM};
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct SignalHandler {
    pub(crate) runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    pub(crate) enable_internal_handler: bool,
    pub(crate) context: ServiceContext,
}

#[cfg(not(target_arch = "wasm32"))]
impl SignalHandler {
    pub(crate) async fn run(self) -> Result<(), io::Error> {
        if let Some(mut signals) = crate::get_external_signal_stream() {
            while let Some(Ok(signal)) = signals
                .recv()
                .with_cancellation_token(self.context.cancellation_token())
                .await
            {
                self.handle_signal(signal).await;
            }
        } else if self.enable_internal_handler {
            #[cfg(unix)]
            // SIGSTP cannot be handled
            // https://www.gnu.org/software/libc/manual/html_node/Job-Control-Signals.html
            let signals = Signals::new([
                Signal::Int,
                Signal::Quit,
                Signal::Abort,
                Signal::Term,
                Signal::Tstp,
                Signal::Cont,
                Signal::Usr1,
                Signal::Usr2,
            ]);
            #[cfg(windows)]
            let signals = Signals::new([Signal::Int]);
            let mut signals =
                signals.inspect_err(|e| error!("error creating signal stream: {e:?}"))?;

            while let Some(Ok(signal)) = signals
                .next()
                .with_cancellation_token(self.context.cancellation_token())
                .await
                .flatten()
            {
                self.handle_signal(signal).await;
            }
        }
        Ok(())
    }

    async fn handle_signal(&self, signal: async_signal::Signal) {
        use crate::{ControlFlow, with_state};

        let on_signal = with_state(|s| s.on_os_signal.lock_mut()(map_signal(signal)));
        if on_signal.await == ControlFlow::Prevent {
            return;
        }
        match signal {
            Signal::Tstp => {
                let _ = self.runtime_command_tx.send(RuntimeCommand::Suspend);
            }
            Signal::Cont => {
                let _ = self.runtime_command_tx.send(RuntimeCommand::Resume);
            }
            Signal::Usr1 | Signal::Usr2 => {
                // SIGUSR1 and SIGUSR2 have no default behavior
            }
            signal => {
                let code = match signal {
                    Signal::Int => proc_exit::SIGINT,
                    Signal::Quit => proc_exit::SIGQUIT,
                    Signal::Abort => proc_exit::SIGABRT,
                    Signal::Term => proc_exit::SIGTERM,
                    _ => unreachable!(),
                };
                let _ = self
                    .runtime_command_tx
                    .send(RuntimeCommand::Terminate(Ok(code)));
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn map_signal(signal: async_signal::Signal) -> crate::OsSignal {
    use async_signal::Signal;
    match signal {
        Signal::Int => crate::OsSignal::Int,
        Signal::Quit => crate::OsSignal::Quit,
        Signal::Abort => crate::OsSignal::Abort,
        Signal::Term => crate::OsSignal::Term,
        Signal::Tstp => crate::OsSignal::Tstp,
        Signal::Cont => crate::OsSignal::Cont,
        Signal::Usr1 => crate::OsSignal::Usr1,
        Signal::Usr2 => crate::OsSignal::Usr2,
        _ => unreachable!(),
    }
}
