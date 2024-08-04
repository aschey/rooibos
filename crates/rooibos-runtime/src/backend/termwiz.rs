use std::fmt::Display;
use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::os::fd::AsRawFd;
use std::time::Duration;

use ratatui::Viewport;
use termwiz::caps::Capabilities;
use termwiz::surface::Change;
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
    title: Option<String>,
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl<W> TerminalSettings<W> {
    fn new<F>(writer: F) -> Self
    where
        F: Fn() -> W + Send + Sync + 'static,
    {
        Self {
            alternate_screen: true,
            viewport: Viewport::default(),
            title: None,
            get_writer: Box::new(writer),
        }
    }
}

impl<W> TerminalSettings<W> {
    pub fn writer(mut self, get_writer: impl Fn() -> W + Send + Sync + 'static) -> Self {
        self.get_writer = Box::new(get_writer);
        self
    }

    pub fn viewport(mut self, viewport: Viewport) -> Self {
        if viewport != Viewport::Fullscreen {
            self.alternate_screen = false;
        }
        self.viewport = viewport;
        self
    }

    pub fn title<T: Display>(mut self, title: T) -> Self {
        self.title = Some(title.to_string());
        self
    }
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self::new(stdout)
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self::new(stderr)
    }
}

impl Default for TermwizBackend<Stdout> {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl Default for TermwizBackend<Stderr> {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl TermwizBackend<Stdout> {
    pub fn stdout() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl TermwizBackend<Stderr> {
    pub fn stderr() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl<W: io::Write> TermwizBackend<W> {
    pub fn new(settings: TerminalSettings<W>) -> Self {
        Self { settings }
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

        if let Some(title) = &self.settings.title {
            terminal.add_change(Change::Title(title.clone()));
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

    fn set_title<T: std::fmt::Display>(
        &self,
        terminal: &mut ratatui::Terminal<Self::TuiBackend>,
        title: T,
    ) -> io::Result<()> {
        terminal
            .backend_mut()
            .buffered_terminal_mut()
            .add_change(Change::Title(title.to_string()));
        Ok(())
    }

    #[cfg(feature = "clipboard")]
    fn set_clipboard<T: Display>(
        &self,
        terminal: &mut ratatui::Terminal<Self::TuiBackend>,
        content: T,
        clipboard_kind: super::ClipboardKind,
    ) -> io::Result<()> {
        use termwiz::escape::osc::Selection;

        let action = termwiz::escape::Action::OperatingSystemCommand(Box::new(
            termwiz::escape::OperatingSystemCommand::SetSelection(
                match clipboard_kind {
                    super::ClipboardKind::Clipboard => Selection::CLIPBOARD,
                    super::ClipboardKind::Primary => Selection::PRIMARY,
                },
                content.to_string(),
            ),
        ));
        terminal
            .backend_mut()
            .buffered_terminal_mut()
            .add_change(action.to_string());
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
