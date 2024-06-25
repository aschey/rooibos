use std::io::{self};

use ratatui::Terminal;
use tokio_util::sync::CancellationToken;

use super::Backend;

pub struct TestBackend {
    width: u16,
    height: u16,
}

impl TestBackend {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
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
        _: tokio::sync::broadcast::Sender<rooibos_dom::Event>,
        _cancellation_token: CancellationToken,
    ) {
    }
}
