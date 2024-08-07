use std::fmt::Display;
use std::io::{self, stderr, stdout, Stderr, Stdout, Write};

use background_service::ServiceContext;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, EventStream, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen, SetTitle,
};
use crossterm::{execute, queue};
use futures_util::StreamExt;
use ratatui::{Terminal, Viewport};
use tap::TapFallible;
use tokio::sync::broadcast;
use tracing::warn;

use super::{Backend, ClipboardKind};

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    focus_change: bool,
    bracketed_paste: bool,
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
        enable_raw_mode()?;
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
        disable_raw_mode()?;

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

    #[cfg(feature = "clipboard")]
    fn set_clipboard<T: Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        content: T,
        clipboard_kind: ClipboardKind,
    ) -> io::Result<()> {
        execute!(
            terminal.backend_mut(),
            SetClipboard::new(&content.to_string(), clipboard_kind)
        )
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.supports_keyboard_enhancement
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        (self.settings.get_writer)().write_all(buf)
    }

    async fn read_input(
        &self,
        term_tx: broadcast::Sender<rooibos_dom::Event>,
        service_context: ServiceContext,
    ) {
        let mut event_reader = EventStream::new().fuse();

        loop {
            tokio::select! {
                _ = service_context.cancelled() => {
                    return;
                }
                event = event_reader.next() => {
                    if let Some(Ok(event)) = event {
                        if let Ok(event) = event.try_into() {
                            let _ = term_tx
                                .send(event)
                                .tap_err(|e| warn!("failed to send event {e:?}"));
                        }
                    } else {
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(feature = "clipboard")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetClipboard {
    // The base64 encoded content for Unix and the raw content for Windows
    payload: String,
    kind: ClipboardKind,
}

#[cfg(feature = "clipboard")]
impl SetClipboard {
    #[cfg(windows)]
    pub fn new(content: &str, kind: ClipboardKind) -> Self {
        Self {
            payload: content.to_string(),
            kind,
        }
    }

    #[cfg(not(windows))]
    pub fn new(content: &str, kind: ClipboardKind) -> Self {
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
            ClipboardKind::Clipboard => "c",
            ClipboardKind::Primary => "p",
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
