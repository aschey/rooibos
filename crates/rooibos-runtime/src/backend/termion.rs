use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::os::fd::AsFd;

use ratatui::{Terminal, Viewport};
use tap::TapFallible;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, IntoAlternateScreen};
use tokio::sync::broadcast;
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use super::Backend;

pub struct TermionBackend<W: Write + AsFd> {
    settings: TerminalSettings<W>,
}

pub struct TerminalSettings<W: Write + AsFd> {
    viewport: Viewport,
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            get_writer: Box::new(stdout),
        }
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
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

impl<W: Write + AsFd> Backend for TermionBackend<W> {
    type TuiBackend =
        ratatui::backend::TermionBackend<MouseTerminal<AlternateScreen<RawTerminal<W>>>>;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>> {
        let terminal = (self.settings.get_writer)()
            .into_raw_mode()?
            .into_alternate_screen()?;
        let terminal = MouseTerminal::from(terminal);
        let mut terminal = Terminal::with_options(
            ratatui::backend::TermionBackend::new(terminal),
            ratatui::TerminalOptions {
                viewport: self.settings.viewport.clone(),
            },
        )?;
        terminal.clear()?;
        Ok(terminal)
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

    async fn read_input(
        &self,
        term_tx: broadcast::Sender<rooibos_dom::Event>,
        cancellation_token: CancellationToken,
    ) {
        let reader = spawn_blocking(move || {
            let stdin = io::stdin();
            for event in stdin.events().flatten() {
                if let Ok(event) = event.try_into() {
                    let _ = term_tx
                        .send(event)
                        .tap_err(|e| warn!("failed to send event {e:?}"));
                }
            }
        });
        tokio::select! {
            _ = reader => {}
            _ = cancellation_token.cancelled() => {}
        }
    }
}
