use std::fmt::Display;
use std::io;
use std::sync::{Arc, Mutex, RwLock};

use crossterm::terminal::disable_raw_mode;
use ratatui::Viewport;
use ratatui::backend::WindowSize;
use rooibos_dom::Event;
use rooibos_terminal::crossterm::CrosstermBackend;
use rooibos_terminal::{self, AsyncInputStream, Backend};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{ArcHandle, SshEventReceiver};

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
    window_size: Arc<RwLock<WindowSize>>,
    inner: CrosstermBackend<ArcHandle>,
}

impl SshBackend {
    pub fn new(handle: ArcHandle, events: SshEventReceiver) -> Self {
        Self::new_with_settings(handle, events, TerminalSettings::default())
    }

    pub fn new_with_settings(
        handle: ArcHandle,
        events: SshEventReceiver,
        settings: TerminalSettings,
    ) -> Self {
        let mut crossterm_settings =
            rooibos_terminal::crossterm::TerminalSettings::from_writer(move || handle.clone())
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
        let window_size = events.window_size;

        // force ANSI escape codes on windows because SSH on Windows uses Unix-style escape codes
        #[cfg(windows)]
        crossterm::ansi_support::force_ansi(true);
        Self {
            event_rx: Mutex::new(Some(events.events)),
            window_size,
            inner,
        }
    }
}

impl Backend for SshBackend {
    type TuiBackend = SshTuiBackend;

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        let inner = self.inner.create_tui_backend()?;
        Ok(SshTuiBackend {
            inner,
            window_size: self.window_size.clone(),
        })
    }

    fn window_size(&self) -> io::Result<ratatui::backend::WindowSize> {
        Ok(*self.window_size.read().unwrap())
    }

    fn setup_terminal(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        self.inner.setup_terminal(&mut backend.inner)
    }

    fn restore_terminal(&self) -> io::Result<()> {
        disable_raw_mode()?;
        self.inner.restore_terminal()
    }

    fn enter_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        self.inner.enter_alt_screen(&mut backend.inner)
    }

    fn leave_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        self.inner.leave_alt_screen(&mut backend.inner)
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.inner.supports_keyboard_enhancement()
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        backend: &mut Self::TuiBackend,
        title: T,
    ) -> io::Result<()> {
        self.inner.set_title(&mut backend.inner, title)
    }

    #[cfg(feature = "clipboard")]
    fn set_clipboard<T: std::fmt::Display>(
        &self,
        backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: rooibos_terminal::ClipboardKind,
    ) -> io::Result<()> {
        self.inner
            .set_clipboard(&mut backend.inner, content, clipboard_kind)
    }

    fn supports_async_input(&self) -> bool {
        true
    }

    fn async_input_stream(&self) -> impl AsyncInputStream {
        let event_rx = self.event_rx.lock().unwrap().take().unwrap();
        ReceiverStream::new(event_rx)
    }
}

pub struct SshTuiBackend {
    inner: ratatui::backend::CrosstermBackend<ArcHandle>,
    window_size: Arc<RwLock<WindowSize>>,
}

impl ratatui::backend::Backend for SshTuiBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        self.inner.draw(content)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor()
    }

    fn get_cursor_position(&mut self) -> io::Result<ratatui::prelude::Position> {
        self.inner.get_cursor_position()
    }

    fn set_cursor_position<P: Into<ratatui::prelude::Position>>(
        &mut self,
        position: P,
    ) -> io::Result<()> {
        self.inner.set_cursor_position(position)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn size(&self) -> io::Result<ratatui::prelude::Size> {
        Ok(self.window_size.read().unwrap().columns_rows)
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(*self.window_size.read().unwrap())
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_up(
        &mut self,
        region: std::ops::Range<u16>,
        line_count: u16,
    ) -> io::Result<()> {
        self.inner.scroll_region_up(region, line_count)
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_down(
        &mut self,
        region: std::ops::Range<u16>,
        line_count: u16,
    ) -> io::Result<()> {
        self.inner.scroll_region_down(region, line_count)
    }
}
