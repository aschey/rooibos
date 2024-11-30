use std::fmt::Display;
use std::io::{self, Stderr, Stdout, Write, stderr, stdout};
use std::os::fd::AsRawFd;
use std::time::Duration;

use ratatui::backend::WindowSize;
use ratatui::layout::Size;
use termwiz::caps::Capabilities;
use termwiz::surface::Change;
use termwiz::terminal::buffered::BufferedTerminal;
use termwiz::terminal::{ScreenSize, SystemTerminal, Terminal};
use tokio::sync::broadcast;
use tracing::warn;

use super::Backend;

pub struct TermwizBackend<W: Write> {
    settings: TerminalSettings<W>,
}

pub struct TerminalSettings<W> {
    alternate_screen: bool,
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

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        let caps = Capabilities::new_from_env().map_err(into_io_error)?;
        let terminal = SystemTerminal::new_with(caps, &io::stdin(), &(self.settings.get_writer)())
            .map_err(into_io_error)?;
        let terminal = BufferedTerminal::new(terminal).map_err(into_io_error)?;

        Ok(ratatui::backend::TermwizBackend::with_buffered_terminal(
            terminal,
        ))
    }

    fn window_size(&self) -> io::Result<WindowSize> {
        let caps = Capabilities::new_from_env().map_err(into_io_error)?;
        let mut terminal =
            SystemTerminal::new_with(caps, &io::stdin(), &(self.settings.get_writer)())
                .map_err(into_io_error)?;
        let ScreenSize {
            cols,
            rows,
            xpixel,
            ypixel,
        } = terminal
            .get_screen_size()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(WindowSize {
            columns_rows: Size {
                width: cols as u16,
                height: rows as u16,
            },
            pixels: Size {
                width: xpixel as u16,
                height: ypixel as u16,
            },
        })
    }

    fn setup_terminal(&self, terminal: &mut ratatui::Terminal<Self::TuiBackend>) -> io::Result<()> {
        let terminal = terminal.backend_mut().buffered_terminal_mut();

        terminal.terminal().set_raw_mode().map_err(into_io_error)?;

        if self.settings.alternate_screen {
            terminal
                .terminal()
                .enter_alternate_screen()
                .map_err(into_io_error)?;
        }

        if let Some(title) = &self.settings.title {
            terminal.add_change(Change::Title(title.clone()));
        }

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

    fn enter_alt_screen(
        &self,
        terminal: &mut ratatui::Terminal<Self::TuiBackend>,
    ) -> io::Result<()> {
        terminal
            .backend_mut()
            .buffered_terminal_mut()
            .terminal()
            .enter_alternate_screen()
            .map_err(into_io_error)?;
        Ok(())
    }

    fn leave_alt_screen(
        &self,
        terminal: &mut ratatui::Terminal<Self::TuiBackend>,
    ) -> io::Result<()> {
        terminal
            .backend_mut()
            .buffered_terminal_mut()
            .terminal()
            .enter_alternate_screen()
            .map_err(into_io_error)?;
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

    fn set_clipboard<T: Display>(
        &self,
        terminal: &mut ratatui::Terminal<Self::TuiBackend>,
        content: T,
        clipboard_kind: super::ClipboardKind,
    ) -> io::Result<()> {
        #[cfg(feature = "clipboard")]
        {
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
        #[cfg(not(feature = "clipboard"))]
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "clipboard feature not enabled",
        ));
    }

    fn supports_async_input(&self) -> bool {
        false
    }

    fn poll_input(
        &self,
        terminal: &mut ratatui::Terminal<Self::TuiBackend>,
        term_tx: &broadcast::Sender<rooibos_dom::Event>,
    ) -> io::Result<()> {
        if let Ok(Some(event)) = terminal
            .backend_mut()
            .buffered_terminal_mut()
            .terminal()
            .poll_input(Some(Duration::ZERO))
        {
            if let Ok(event) = event.try_into() {
                let _ = term_tx
                    .send(event)
                    .inspect_err(|e| warn!("error sending input: {e:?}"));
            }
        }
        Ok(())
    }

    fn async_input_stream(&self) -> impl crate::AsyncInputStream {
        futures_util::stream::empty()
    }
}

fn into_io_error(error: termwiz::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, error.to_string())
}
