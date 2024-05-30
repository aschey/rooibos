use std::io::{self};

use ratatui::Terminal;

use super::Backend;
use crate::SignalMode;

pub struct TestBackend {
    width: u16,
    height: u16,
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

    fn write_all(&self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    async fn read_input(
        &self,
        _: tokio::sync::mpsc::Sender<SignalMode>,
        _: tokio::sync::broadcast::Sender<rooibos_dom::Event>,
    ) {
    }
}
