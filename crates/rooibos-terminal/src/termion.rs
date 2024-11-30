use std::io::{self, Stderr, Stdout, Write, stderr, stdout};
use std::os::fd::AsFd;

use ratatui::Terminal;
use ratatui::backend::WindowSize;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, IntoAlternateScreen};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::task::spawn_blocking;
use tokio_stream::wrappers::ReceiverStream;

use super::Backend;
use crate::AsyncInputStream;

pub struct TermionBackend<W: Write + AsFd> {
    settings: TerminalSettings<W>,
}

pub struct TerminalSettings<W: Write + AsFd> {
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self {
            get_writer: Box::new(stdout),
        }
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self {
            get_writer: Box::new(stderr),
        }
    }
}

impl Default for TermionBackend<Stdout> {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl Default for TermionBackend<Stderr> {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl TermionBackend<Stdout> {
    pub fn stdout() -> Self {
        Self::default()
    }
}

impl TermionBackend<Stderr> {
    pub fn stderr() -> Self {
        Self::default()
    }
}

impl<W: Write + AsFd> Backend for TermionBackend<W> {
    type TuiBackend =
        ratatui::backend::TermionBackend<MouseTerminal<AlternateScreen<RawTerminal<W>>>>;

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        let terminal = (self.settings.get_writer)()
            .into_raw_mode()?
            .into_alternate_screen()?;
        let terminal = MouseTerminal::from(terminal);
        Ok(ratatui::backend::TermionBackend::new(terminal))
    }

    fn window_size(&self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: termion::terminal_size()?.into(),
            pixels: termion::terminal_size_pixels()?.into(),
        })
    }

    fn setup_terminal(&self, _terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        Ok(())
    }

    fn restore_terminal(&self) -> io::Result<()> {
        Ok(())
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        false
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        (self.settings.get_writer)().write_all(buf)
    }

    fn enter_alt_screen(&self, _terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "termion backend does not support alt screen toggle",
        ))
    }

    fn leave_alt_screen(&self, _terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "termion backend does not support alt screen toggle",
        ))
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        _terminal: &mut Terminal<Self::TuiBackend>,
        _title: T,
    ) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "termion backend does not support setting the window title",
        ))
    }

    fn set_clipboard<T: std::fmt::Display>(
        &self,
        _terminal: &mut Terminal<Self::TuiBackend>,
        _content: T,
        _clipboard_kind: super::ClipboardKind,
    ) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "termion backend does not support setting the clipboard",
        ))
    }

    fn async_input_stream(&self) -> impl AsyncInputStream {
        let (tx, rx) = mpsc::channel(128);
        spawn_blocking(move || {
            let stdin = io::stdin();
            for event in stdin.events().flatten() {
                let event: Result<rooibos_dom::Event, _> = event.try_into();
                if let Ok(event) = event {
                    if let Err(TrySendError::Closed(_)) = tx.try_send(event) {
                        return;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }
}
