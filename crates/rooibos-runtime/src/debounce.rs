use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::wasm_compat;

pub(crate) struct Debouncer<T> {
    pending: Option<T>,
    update_tx: mpsc::Sender<()>,
    ready_rx: mpsc::Receiver<()>,
    has_value: Arc<AtomicBool>,
}

impl<T> Debouncer<T> {
    pub(crate) fn new(debounce_time: Duration) -> Self {
        let (ready_tx, ready_rx) = mpsc::channel(32);
        let (update_tx, mut update_rx) = mpsc::channel(32);
        let has_value = Arc::new(AtomicBool::new(false));

        wasm_compat::spawn_local({
            let has_value = has_value.clone();
            async move {
                loop {
                    let timeout = wasm_compat::sleep(debounce_time);
                    tokio::select! {
                        _ = update_rx.recv() => {},
                        _ = timeout => {
                            if has_value.load(Ordering::Relaxed) {
                                ready_tx.send(()).await.unwrap();
                            }
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
        }
    }

    pub(crate) async fn update(&mut self, value: T) {
        self.pending = Some(value);
        self.has_value.store(true, Ordering::Relaxed);
        self.update_tx.send(()).await.unwrap();
    }

    pub(crate) async fn next_value(&mut self) -> T {
        self.ready_rx.recv().await.unwrap();
        self.has_value.store(false, Ordering::Relaxed);
        self.pending.take().unwrap()
    }
}
