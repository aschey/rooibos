use std::fmt::Display;
use std::io::{self, Stderr, Stdout, Write, stderr, stdout};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, EventStream, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
    supports_keyboard_enhancement,
};
use crossterm::{execute, queue};
use ratatui::{Terminal, Viewport};
use tokio_stream::StreamExt as _;

use super::Backend;
use crate::AsyncInputStream;

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    focus_change: bool,
    bracketed_paste: bool,
    raw_mode: bool,
    viewport: Viewport,
    title: Option<String>,
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalSettings<Stdout> {
    pub fn new() -> Self {
        Self::from_writer(stdout)
    }
}

impl TerminalSettings<Stderr> {
    pub fn new() -> Self {
        Self::from_writer(stderr)
    }
}

impl<W> TerminalSettings<W> {
    pub fn from_writer<F>(get_writer: F) -> Self
    where
        F: Fn() -> W + Send + Sync + 'static,
    {
        Self {
            alternate_screen: true,
            raw_mode: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            focus_change: true,
            bracketed_paste: true,
            viewport: Viewport::default(),
            title: None,
            get_writer: Box::new(get_writer),
        }
    }

    pub fn alternate_screen(mut self, alternate_screen: bool) -> Self {
        self.alternate_screen = alternate_screen;
        self
    }

    pub fn mouse_capture(mut self, mouse_capture: bool) -> Self {
        self.mouse_capture = mouse_capture;
        self
    }

    pub fn focus_change(mut self, focus_change: bool) -> Self {
        self.focus_change = focus_change;
        self
    }

    pub fn bracketed_paste(mut self, bracketed_paste: bool) -> Self {
        self.bracketed_paste = bracketed_paste;
        self
    }

    pub fn raw_mode(mut self, raw_mode: bool) -> Self {
        self.raw_mode = raw_mode;
        self
    }

    pub fn viewport(mut self, viewport: Viewport) -> Self {
        if viewport != Viewport::Fullscreen {
            self.alternate_screen = false;
        }
        self.viewport = viewport;
        self
    }

    pub fn keyboard_enhancement(mut self, keyboard_enhancement: bool) -> Self {
        self.keyboard_enhancement = keyboard_enhancement;
        self
    }

    pub fn title<T: Display>(mut self, title: T) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn writer(mut self, get_writer: impl Fn() -> W + Send + Sync + 'static) -> Self {
        self.get_writer = Box::new(get_writer);
        self
    }
}

pub struct CrosstermBackend<W: Write> {
    settings: TerminalSettings<W>,
    supports_keyboard_enhancement: bool,
}

impl<W: Write> CrosstermBackend<W> {
    pub fn new(settings: TerminalSettings<W>) -> Self {
        Self {
            supports_keyboard_enhancement: if settings.keyboard_enhancement {
                supports_keyboard_enhancement().unwrap_or(false)
            } else {
                false
            },
            settings,
        }
    }
}

impl Default for CrosstermBackend<Stdout> {
    fn default() -> Self {
        Self::new(TerminalSettings::default())
    }
}

impl Default for CrosstermBackend<Stderr> {
    fn default() -> Self {
        Self::new(TerminalSettings::default())
    }
}

impl CrosstermBackend<Stdout> {
    pub fn stdout() -> Self {
        Self::default()
    }
}

impl CrosstermBackend<Stderr> {
    pub fn stderr() -> Self {
        Self::default()
    }
}

impl<W: Write> Backend for CrosstermBackend<W> {
    type TuiBackend = ratatui::backend::CrosstermBackend<W>;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>> {
        let mut writer = (self.settings.get_writer)();
        if self.settings.raw_mode {
            enable_raw_mode()?;
        }

        queue!(writer, Hide)?;
        if self.settings.alternate_screen {
            queue!(writer, EnterAlternateScreen)?;
        }
        if self.settings.mouse_capture {
            queue!(writer, EnableMouseCapture)?;
        }
        if self.settings.focus_change {
            queue!(writer, EnableFocusChange)?;
        }
        if self.settings.bracketed_paste {
            queue!(writer, EnableBracketedPaste)?;
        }
        if self.supports_keyboard_enhancement {
            queue!(
                writer,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
            )?;
        }
        if let Some(title) = &self.settings.title {
            queue!(writer, SetTitle(title))?;
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
        if self.settings.raw_mode {
            disable_raw_mode()?;
        }
        if self.supports_keyboard_enhancement {
            queue!(writer, PopKeyboardEnhancementFlags)?;
        }
        if self.settings.mouse_capture {
            queue!(writer, DisableMouseCapture)?;
        }
        if self.settings.focus_change {
            queue!(writer, DisableFocusChange)?;
        }
        if self.settings.bracketed_paste {
            queue!(writer, DisableBracketedPaste)?;
        }
        if self.settings.alternate_screen {
            queue!(writer, LeaveAlternateScreen)?;
        }

        queue!(writer, Show)?;
        writer.flush()?;

        Ok(())
    }

    fn enter_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        execute!(terminal.backend_mut(), EnterAlternateScreen)
    }

    fn leave_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        title: T,
    ) -> io::Result<()> {
        execute!(terminal.backend_mut(), SetTitle(title))
    }

    fn set_clipboard<T: Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        content: T,
        clipboard_kind: super::ClipboardKind,
    ) -> io::Result<()> {
        #[cfg(feature = "clipboard")]
        return execute!(
            terminal.backend_mut(),
            SetClipboard::new(&content.to_string(), clipboard_kind)
        );
        #[cfg(not(feature = "clipboard"))]
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "clipboard feature not enabled",
        ));
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.supports_keyboard_enhancement
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        (self.settings.get_writer)().write_all(buf)
    }

    fn async_input_stream(&self) -> impl AsyncInputStream {
        let event_reader = EventStream::new().fuse();
        event_reader.filter_map(move |e| {
            if let Ok(e) = e {
                let e: Result<rooibos_dom::Event, _> = e.try_into();
                return e.ok();
            }
            None
        })
    }
}

#[cfg(feature = "clipboard")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetClipboard {
    // The base64 encoded content for Unix and the raw content for Windows
    payload: String,
    kind: super::ClipboardKind,
}

#[cfg(feature = "clipboard")]
impl SetClipboard {
    #[cfg(windows)]
    pub fn new(content: &str, kind: super::ClipboardKind) -> Self {
        Self {
            payload: content.to_string(),
            kind,
        }
    }

    #[cfg(not(windows))]
    pub fn new(content: &str, kind: super::ClipboardKind) -> Self {
        use base64::prelude::*;
        Self {
            payload: BASE64_STANDARD.encode(content),
            kind,
        }
    }
}

#[cfg(feature = "clipboard")]
impl crossterm::Command for SetClipboard {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        // Send an OSC 52 set command: https://terminalguide.namepad.de/seq/osc-52/
        let kind = match &self.kind {
            super::ClipboardKind::Clipboard => "c",
            super::ClipboardKind::Primary => "p",
        };
        write!(f, "\x1b]52;{};{}\x1b\\", kind, &self.payload)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> io::Result<()> {
        clipboard_win::set_clipboard_string(&self.payload)
            .map_err(|e| io::Error::from_raw_os_error(e.raw_code()))
    }

    #[cfg(windows)]
    fn is_ansi_code_supported(&self) -> bool {
        false
    }
}
