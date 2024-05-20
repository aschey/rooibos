use std::io::{self, stderr, stdout, Stderr, Stdout, Write};

use crossterm::cursor::Show;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::queue;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use futures_util::StreamExt;
use ratatui::{Terminal, Viewport};
use tap::TapFallible;
use tracing::warn;

use super::Backend;

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    viewport: Viewport,
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            viewport: Viewport::default(),
            get_writer: Box::new(stdout),
        }
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            viewport: Viewport::default(),
            get_writer: Box::new(stderr),
        }
    }
}

impl<W> TerminalSettings<W> {
    pub fn alternate_screen(mut self, alternate_screen: bool) -> Self {
        self.alternate_screen = alternate_screen;
        self
    }

    pub fn mouse_capture(mut self, mouse_capture: bool) -> Self {
        self.mouse_capture = mouse_capture;
        self
    }

    pub fn keyboard_enhancement(mut self, keyboard_enhancement: bool) -> Self {
        self.keyboard_enhancement = keyboard_enhancement;
        self
    }

    pub fn writer(mut self, get_writer: impl Fn() -> W + Send + Sync + 'static) -> Self {
        self.get_writer = Box::new(get_writer);
        self
    }
}

pub struct CrosstermBackend<W: Write> {
    settings: TerminalSettings<W>,
}

impl<W: Write> CrosstermBackend<W> {
    pub fn new(settings: TerminalSettings<W>) -> Self {
        Self { settings }
    }
}

impl Default for CrosstermBackend<Stdout> {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl Default for CrosstermBackend<Stderr> {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl<W: Write> Backend for CrosstermBackend<W> {
    type TuiBackend = ratatui::backend::CrosstermBackend<W>;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>> {
        let mut writer = (self.settings.get_writer)();
        enable_raw_mode()?;
        if self.settings.alternate_screen {
            queue!(writer, EnterAlternateScreen)?;
        }
        if self.settings.mouse_capture {
            queue!(writer, EnableMouseCapture)?;
        }
        if self.settings.keyboard_enhancement {
            queue!(
                writer,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
            )?;
        }
        writer.flush()?;

        let mut terminal = Terminal::with_options(
            ratatui::backend::CrosstermBackend::new(writer),
            ratatui::TerminalOptions {
                viewport: self.settings.viewport.clone(),
            },
        )?;

        terminal.clear()?;
        Ok(terminal)
    }

    fn restore_terminal(&self) -> io::Result<()> {
        let mut writer = (self.settings.get_writer)();

        if self.settings.keyboard_enhancement {
            queue!(writer, PopKeyboardEnhancementFlags)?;
        }

        if self.settings.mouse_capture {
            queue!(writer, DisableMouseCapture)?;
        }

        if self.settings.alternate_screen {
            queue!(writer, LeaveAlternateScreen)?;
        }
        queue!(writer, Show)?;
        writer.flush()?;
        disable_raw_mode()?;

        Ok(())
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        if self.settings.keyboard_enhancement {
            supports_keyboard_enhancement().unwrap_or(false)
        } else {
            false
        }
    }

    async fn read_input(
        quit_tx: tokio::sync::mpsc::Sender<()>,
        term_tx: tokio::sync::broadcast::Sender<rooibos_dom::Event>,
    ) {
        let mut event_reader = crossterm::event::EventStream::new().fuse();
        while let Some(Ok(event)) = event_reader.next().await {
            if let event::Event::Key(key_event) = event {
                let KeyEvent {
                    code, modifiers, ..
                } = key_event;

                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    let _ = quit_tx
                        .send(())
                        .await
                        .tap_err(|e| warn!("error sending quit signal {e:?}"));
                    break;
                }
            }
            term_tx.send(event.into()).ok();
        }
    }
}
