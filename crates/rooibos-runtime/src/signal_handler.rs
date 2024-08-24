use std::io;

use async_signal::{Signal, Signals};
use background_service::ServiceContext;
use futures_cancel::FutureExt;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tracing::error;

use crate::RuntimeCommand;

pub(crate) struct SignalHandler {
    pub(crate) runtime_command_tx: broadcast::Sender<RuntimeCommand>,
    pub(crate) enable_internal_handler: bool,
    pub(crate) context: ServiceContext,
}

impl SignalHandler {
    pub(crate) async fn run(self) -> Result<(), io::Error> {
        if let Some(mut signals) = crate::get_external_signal_stream() {
            while let Ok(Ok(signal)) = signals.recv().cancel_with(self.context.cancelled()).await {
                self.handle_signal(signal);
            }
        } else if self.enable_internal_handler {
            #[cfg(unix)]
            // SIGSTP cannot be handled
            // https://www.gnu.org/software/libc/manual/html_node/Job-Control-Signals.html
            let signals = Signals::new([
                Signal::Term,
                Signal::Quit,
                Signal::Int,
                Signal::Tstp,
                Signal::Cont,
            ]);
            #[cfg(windows)]
            let signals = Signals::new([Signal::Int]);
            let mut signals =
                signals.inspect_err(|e| error!("error creating signal stream: {e:?}"))?;

            while let Ok(Some(Ok(signal))) =
                signals.next().cancel_with(self.context.cancelled()).await
            {
                self.handle_signal(signal);
            }
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_signal(&self, signal: async_signal::Signal) {
        use async_signal::Signal;
        match signal {
            Signal::Tstp => {
                let _ = self.runtime_command_tx.send(RuntimeCommand::Suspend);
            }
            Signal::Cont => {
                let _ = self.runtime_command_tx.send(RuntimeCommand::Resume);
            }
            _ => {
                let _ = self.runtime_command_tx.send(RuntimeCommand::Terminate);
            }
        }
    }
}
