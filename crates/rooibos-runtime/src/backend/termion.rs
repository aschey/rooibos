use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::os::fd::AsFd;
use std::pin::Pin;

use futures_util::Future;
use ratatui::{Terminal, Viewport};
use tap::TapFallible;
use termion::event;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, IntoAlternateScreen};
use tokio::task::spawn_blocking;
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

    fn read_input(
        &self,
        quit_tx: tokio::sync::mpsc::Sender<()>,
        term_tx: tokio::sync::broadcast::Sender<rooibos_dom::Event>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            spawn_blocking(move || {
                let stdin = io::stdin();
                for event in stdin.events().flatten() {
                    if let event::Event::Key(key) = event {
                        if key == event::Key::Ctrl('c') {
                            let _ = quit_tx
                                .try_send(())
                                .tap_err(|e| warn!("error sending quit signal {e:?}"));
                            break;
                        }
                    }
                    term_tx.send(event.into()).ok();
                }
            })
            .await
            .unwrap();
        })
    }
}
