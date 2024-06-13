use std::io;
use std::sync::Mutex;

use ratatui::Terminal;
use rooibos_dom::Event;
use rooibos_runtime::backend::Backend;
use tap::TapFallible;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

pub struct SshBackend<B: Backend> {
    event_rx: Mutex<Option<mpsc::Receiver<Event>>>,
    inner: B,
}

impl<B: Backend> SshBackend<B> {
    pub fn new(inner: B, event_rx: mpsc::Receiver<Event>) -> Self {
        // force ANSI escape codes on windows because SSH on Windows uses Unix-style escape codes
        #[cfg(all(windows, feature = "crossterm"))]
        crossterm::ansi_support::force_ansi(true);
        Self {
            event_rx: Mutex::new(Some(event_rx)),
            inner,
        }
    }
}

impl<B: Backend> Backend for SshBackend<B> {
    type TuiBackend = B::TuiBackend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>> {
        self.inner.setup_terminal()
    }

    fn restore_terminal(&self) -> io::Result<()> {
        self.inner.restore_terminal()
    }

    fn enter_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        self.inner.enter_alt_screen(terminal)
    }

    fn leave_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        self.inner.leave_alt_screen(terminal)
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.inner.supports_keyboard_enhancement()
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        title: T,
    ) -> io::Result<()> {
        self.inner.set_title(terminal, title)
    }

    fn supports_async_input(&self) -> bool {
        true
    }

    async fn read_input(&self, term_tx: broadcast::Sender<rooibos_dom::Event>) {
        let mut event_rx = self.event_rx.lock().unwrap().take().unwrap();
        while let Some(event) = event_rx.recv().await {
            let _ = term_tx
                .send(event)
                .tap_err(|e| warn!("failed to send event {e:?}"));
        }
    }
}
