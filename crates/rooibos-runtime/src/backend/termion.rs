use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::os::fd::AsFd;

use ratatui::{Terminal, Viewport};
use tap::TapFallible;
use termion::event;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, IntoAlternateScreen};
use tokio::sync::{broadcast, mpsc};
use tokio::task::spawn_blocking;
use tracing::warn;

use super::Backend;
use crate::SignalMode;

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

    fn enter_alt_screen(&self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "termion backend does not support alt screen toggle",
        ))
    }

    fn leave_alt_screen(&self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "termion backend does not support alt screen toggle",
        ))
    }

    async fn read_input(
        &self,
        signal_tx: mpsc::Sender<SignalMode>,
        term_tx: broadcast::Sender<rooibos_dom::Event>,
    ) {
        spawn_blocking(move || {
            let stdin = io::stdin();
            for event in stdin.events().flatten() {
                if let event::Event::Key(key) = event {
                    match key {
                        event::Key::Ctrl('c') => {
                            let _ = signal_tx
                                .try_send(SignalMode::Terminate)
                                .tap_err(|e| warn!("error sending quit signal {e:?}"));
                            break;
                        }
                        event::Key::Ctrl('z') => {
                            if cfg!(unix) {
                                let _ = signal_tx
                                    .try_send(SignalMode::Suspend)
                                    .tap_err(|e| warn!("error sending quit signal {e:?}"));
                                continue;
                            }
                        }
                        _ => {}
                    }
                }
                let _ = term_tx
                    .send(event.into())
                    .tap_err(|e| warn!("error sending terminal event {e:?}"));
            }
        })
        .await
        .unwrap();
    }
}
