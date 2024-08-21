use std::fmt::Display;
use std::io;
use std::sync::Mutex;

use crossterm::terminal::disable_raw_mode;
use futures_cancel::FutureExt;
use ratatui::{Terminal, Viewport};
use rooibos_dom::Event;
use rooibos_runtime::backend::crossterm::CrosstermBackend;
use rooibos_runtime::backend::{self, Backend};
use rooibos_runtime::ServiceContext;
use tap::TapFallible;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

use crate::ArcHandle;

pub struct TerminalSettings {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    focus_change: bool,
    bracketed_paste: bool,
    viewport: Viewport,
    title: Option<String>,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalSettings {
    pub fn new() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            focus_change: true,
            bracketed_paste: true,
            viewport: Viewport::default(),
            title: None,
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
}

pub struct SshBackend {
    event_rx: Mutex<Option<mpsc::Receiver<Event>>>,
    inner: CrosstermBackend<ArcHandle>,
}

impl SshBackend {
    pub fn new(handle: ArcHandle, event_rx: mpsc::Receiver<Event>) -> Self {
        Self::new_with_settings(handle, event_rx, TerminalSettings::default())
    }

    pub fn new_with_settings(
        handle: ArcHandle,
        event_rx: mpsc::Receiver<Event>,
        settings: TerminalSettings,
    ) -> Self {
        let mut crossterm_settings =
            backend::crossterm::TerminalSettings::from_writer(move || handle.clone())
                .raw_mode(false)
                .alternate_screen(settings.alternate_screen)
                .bracketed_paste(settings.bracketed_paste)
                .focus_change(settings.focus_change)
                .keyboard_enhancement(settings.keyboard_enhancement)
                .mouse_capture(settings.mouse_capture);
        if let Some(title) = settings.title {
            crossterm_settings = crossterm_settings.title(title);
        }
        let inner = CrosstermBackend::new(crossterm_settings);
        // force ANSI escape codes on windows because SSH on Windows uses Unix-style escape codes
        #[cfg(windows)]
        crossterm::ansi_support::force_ansi(true);
        Self {
            event_rx: Mutex::new(Some(event_rx)),
            inner,
        }
    }
}

impl Backend for SshBackend {
    type TuiBackend = <CrosstermBackend<ArcHandle> as Backend>::TuiBackend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>> {
        self.inner.setup_terminal()
    }

    fn restore_terminal(&self) -> io::Result<()> {
        disable_raw_mode()?;
        self.inner.restore_terminal()
    }

    fn enter_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        self.inner.enter_alt_screen(terminal)
    }

    fn leave_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        self.inner.leave_alt_screen(terminal)
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.inner.supports_keyboard_enhancement()
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        title: T,
    ) -> io::Result<()> {
        self.inner.set_title(terminal, title)
    }

    #[cfg(feature = "clipboard")]
    fn set_clipboard<T: std::fmt::Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        content: T,
        clipboard_kind: rooibos_runtime::backend::ClipboardKind,
    ) -> io::Result<()> {
        self.inner.set_clipboard(terminal, content, clipboard_kind)
    }

    fn supports_async_input(&self) -> bool {
        true
    }

    async fn read_input(
        &self,
        term_tx: broadcast::Sender<rooibos_dom::Event>,
        context: ServiceContext,
    ) {
        let mut event_rx = self.event_rx.lock().unwrap().take().unwrap();
        while let Ok(event) = event_rx.recv().cancel_with(context.cancelled()).await {
            if let Some(event) = event {
                let _ = term_tx
                    .send(event)
                    .tap_err(|e| warn!("failed to send event {e:?}"));
            }
        }
    }
}
