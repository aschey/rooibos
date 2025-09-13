use std::io;

use ratatui::backend::WindowSize;
use ratatui::layout::Size;
use rooibos_terminal::{AsyncInputStream, Backend, ClipboardKind};
use stream_cancel::StreamExt;
use tokio::sync::broadcast;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;

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

    fn create_tui_backend(&self) -> std::io::Result<Self::TuiBackend> {
        Ok(ratatui::backend::TestBackend::new(self.width, self.height))
    }

    fn setup_terminal(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        Ok(())
    }

    fn restore_terminal(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        true
    }

    fn window_size(&self, _backend: &mut Self::TuiBackend) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: Size {
                width: self.width,
                height: self.height,
            },
            pixels: Size::default(),
        })
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        _backend: &mut Self::TuiBackend,
        _title: T,
    ) -> io::Result<()> {
        Ok(())
    }

    fn set_clipboard<T: std::fmt::Display>(
        &self,
        _backend: &mut Self::TuiBackend,
        _content: T,
        _clipboard_kind: ClipboardKind,
    ) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn enter_alt_screen(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        Ok(())
    }

    fn leave_alt_screen(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        Ok(())
    }

    fn async_input_stream(&self, cancellation_token: CancellationToken) -> impl AsyncInputStream {
        let rx = self.event_tx.subscribe();
        BroadcastStream::new(rx)
            .filter_map(|e| e.ok())
            .take_until_if(async move {
                cancellation_token.cancelled().await;
                true
            })
    }
}
