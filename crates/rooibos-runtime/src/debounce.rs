use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rooibos_reactive::spawn_local;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use crate::wasm_compat;

#[derive(Debug)]
pub(crate) struct Debouncer<T> {
    pending: Option<T>,
    update_tx: mpsc::Sender<()>,
    ready_rx: mpsc::Receiver<()>,
    has_value: Arc<AtomicBool>,
    cancellation_token: CancellationToken,
}

impl<T> Drop for Debouncer<T> {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

impl<T> Debouncer<T> {
    pub(crate) fn new(debounce_time: Duration) -> Self {
        let (ready_tx, ready_rx) = mpsc::channel(32);
        let (update_tx, mut update_rx) = mpsc::channel(32);
        let has_value = Arc::new(AtomicBool::new(false));
        let cancellation_token = CancellationToken::new();

        spawn_local({
            let has_value = has_value.clone();
            let cancellation_token = cancellation_token.clone();

            async move {
                loop {
                    let timeout = wasm_compat::sleep(debounce_time);
                    tokio::select! {
                        Some(_) = update_rx.recv() => {},
                        _ = timeout => {
                            if has_value.load(Ordering::Relaxed) {
                                let _ = ready_tx
                                    .send(())
                                    .await
                                    .inspect_err(|e| error!("error sending ready signal: {e:?}"));
                            }
                        }
                        _ = cancellation_token.cancelled() => {
                            return;
                        }
                    }
                }
            }
        });

        Self {
            pending: None,
            update_tx,
            ready_rx,
            has_value,
            cancellation_token,
        }
    }

    pub(crate) async fn update(&mut self, value: T) -> Result<(), mpsc::error::SendError<()>> {
        self.pending = Some(value);
        self.has_value.store(true, Ordering::Relaxed);
        self.update_tx.send(()).await?;
        Ok(())
    }

    pub(crate) async fn next_value(&mut self) -> Option<T> {
        self.ready_rx.recv().await?;
        self.has_value.store(false, Ordering::Relaxed);
        self.pending.take()
    }
}
