use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::os::fd::AsRawFd;
use std::time::Duration;

use ratatui::Viewport;
use termwiz::caps::Capabilities;
use termwiz::terminal::buffered::BufferedTerminal;
use termwiz::terminal::{SystemTerminal, Terminal};
use tokio::sync::broadcast;

use super::Backend;

pub struct TermwizBackend<W: Write> {
    settings: TerminalSettings<W>,
}

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    viewport: Viewport,
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            viewport: Viewport::default(),
            get_writer: Box::new(stdout),
        }
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            viewport: Viewport::default(),
            get_writer: Box::new(stderr),
        }
    }
}

impl Default for TermwizBackend<Stdout> {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl<W: Write + AsRawFd> Backend for TermwizBackend<W> {
    type TuiBackend = ratatui::backend::TermwizBackend;

    fn setup_terminal(&self) -> io::Result<ratatui::terminal::Terminal<Self::TuiBackend>> {
        let caps = Capabilities::new_from_env().unwrap();
        let terminal =
            SystemTerminal::new_with(caps, &io::stdin(), &(self.settings.get_writer)()).unwrap();
        let mut terminal = BufferedTerminal::new(terminal).unwrap();
        terminal.terminal().set_raw_mode().unwrap();

        if self.settings.alternate_screen {
            terminal.terminal().enter_alternate_screen().unwrap();
        }

        let terminal = ratatui::terminal::Terminal::with_options(
            ratatui::backend::TermwizBackend::with_buffered_terminal(terminal),
            ratatui::TerminalOptions {
                viewport: self.settings.viewport.clone(),
            },
        )?;
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

    fn enter_alt_screen(
        &self,
        terminal: &mut ratatui::terminal::Terminal<Self::TuiBackend>,
    ) -> io::Result<()> {
        terminal
            .backend_mut()
            .buffered_terminal_mut()
            .terminal()
            .enter_alternate_screen()
            .unwrap();
        Ok(())
    }

    fn leave_alt_screen(
        &self,
        terminal: &mut ratatui::terminal::Terminal<Self::TuiBackend>,
    ) -> io::Result<()> {
        terminal
            .backend_mut()
            .buffered_terminal_mut()
            .terminal()
            .enter_alternate_screen()
            .unwrap();
        Ok(())
    }

    fn supports_async_input(&self) -> bool {
        false
    }

    fn poll_input(
        &self,
        terminal: &mut ratatui::terminal::Terminal<Self::TuiBackend>,
        term_tx: &broadcast::Sender<rooibos_dom::Event>,
    ) -> io::Result<()> {
        if let Ok(Some(event)) = terminal
            .backend_mut()
            .buffered_terminal_mut()
            .terminal()
            .poll_input(Some(Duration::ZERO))
        {
            if let Ok(event) = event.try_into() {
                let _ = term_tx.send(event);
            }
        }
        Ok(())
    }
}
