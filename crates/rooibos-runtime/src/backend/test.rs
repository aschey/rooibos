use std::io;

use background_service::ServiceContext;
use ratatui::Terminal;
use tokio::sync::broadcast;

use super::Backend;

pub struct TestBackend {
    width: u16,
    height: u16,
    event_tx: broadcast::Sender<rooibos_dom::Event>,
}

impl TestBackend {
    pub fn new(width: u16, height: u16) -> Self {
        let (event_tx, _) = broadcast::channel(32);
        Self {
            width,
            height,
            event_tx,
        }
    }

    pub fn event_tx(&self) -> broadcast::Sender<rooibos_dom::Event> {
        self.event_tx.clone()
    }
}

impl Backend for TestBackend {
    type TuiBackend = ratatui::backend::TestBackend;

    fn setup_terminal(&self) -> std::io::Result<ratatui::prelude::Terminal<Self::TuiBackend>> {
        Terminal::new(ratatui::backend::TestBackend::new(self.width, self.height))
    }

    fn restore_terminal(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        true
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        _terminal: &mut Terminal<Self::TuiBackend>,
        _title: T,
    ) -> io::Result<()> {
        Ok(())
    }

    #[cfg(feature = "clipboard")]
    fn set_clipboard<T: std::fmt::Display>(
        &self,
        _terminal: &mut Terminal<Self::TuiBackend>,
        _content: T,
        _clipboard_kind: super::ClipboardKind,
    ) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn enter_alt_screen(&self, _terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        Ok(())
    }

    fn leave_alt_screen(&self, _terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        Ok(())
    }

    async fn read_input(
        &self,
        tx: broadcast::Sender<rooibos_dom::Event>,
        service_context: ServiceContext,
    ) {
        let mut rx = self.event_tx.subscribe();
        loop {
            tokio::select! {
                _ = service_context.cancelled() => {
                    return;
                }
                Ok(event) = rx.recv() => {
                    tx.send(event).unwrap();
                }
            }
        }
    }
}
