use std::collections::HashMap;
use std::fmt::Display;
use std::io::{self};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use futures::executor::block_on;
use ratatui::Viewport;
use ratatui::backend::WindowSize;
use ratatui::layout::{Position, Size};
use rooibos_dom::Event;
use rooibos_runtime::CancellationToken;
use rooibos_terminal::termina::{TerminaBackend, tui};
use rooibos_terminal::{self, AsyncInputStream, Backend};
use stream_cancel::StreamExt;
use termina::escape::csi::{self, Csi, Device, Sgr};
use termina::escape::dcs::{Dcs, DcsResponse};
use termina::style::ColorSpec;
use termina::{PlatformTerminal, Terminal};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tui_theme::ColorPalette;
use tui_theme::profile::{DetectorSettings, IsTerminal, QueryTerminal, TermProfile, TermVars};

use crate::{ArcHandle, SshParams};

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
    inner: TerminaBackend<ArcHandle>,
    profile: TermProfile,
}

struct SshTerminal<'a> {
    handle: ArcHandle,
    query_events: &'a mut mpsc::Receiver<termina::Event>,
}

impl QueryTerminal for SshTerminal<'_> {
    fn setup(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn read_event(&mut self) -> io::Result<tui_theme::profile::Event> {
        match block_on(timeout(
            Duration::from_millis(100),
            self.query_events.recv(),
        )) {
            Ok(Some(event)) => Ok(match event {
                termina::Event::Dcs(Dcs::Response {
                    value: DcsResponse::GraphicRendition(sgrs),
                    ..
                }) => sgrs
                    .iter()
                    .find_map(|s| {
                        if let Sgr::Background(ColorSpec::TrueColor(rgb)) = s {
                            tui_theme::profile::Event::BackgroundColor(tui_theme::profile::Rgb {
                                red: rgb.red,
                                green: rgb.green,
                                blue: rgb.blue,
                            })
                            .into()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(tui_theme::profile::Event::Other),
                termina::Event::Csi(Csi::Device(Device::DeviceAttributes(()))) => {
                    tui_theme::profile::Event::DeviceAttributes
                }
                _ => tui_theme::profile::Event::Other,
            }),
            Err(_) | Ok(None) => Ok(tui_theme::profile::Event::TimedOut),
        }
    }
}

impl io::Write for SshTerminal<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.handle.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
    }
}

impl SshBackend {
    pub async fn new(params: SshParams) -> io::Result<Self> {
        Self::new_with_settings(params, TerminalSettings::default()).await
    }

    pub async fn new_with_settings(
        mut params: SshParams,
        settings: TerminalSettings,
    ) -> io::Result<Self> {
        use io::Write;
        let mut handle = params.handle.clone();
        let (profile, supports_keyboard_enhancement) = tokio::task::spawn_blocking(move || {
            let mut supports_keyboard_enhancement = false;

            let profile = TermProfile::detect_with_vars(TermVars::from_source(
                &HashMap::from_iter([("TERM", params.term.as_str())]),
                &ForceTerminal,
                DetectorSettings::new()
                    .query_terminal(SshTerminal {
                        handle: handle.clone(),
                        query_events: &mut params.events.query_events,
                    })
                    .enable_terminfo(true)
                    .enable_tmux_info(false),
            ));

            if settings.keyboard_enhancement {
                write!(
                    handle,
                    "{}{}",
                    Csi::Keyboard(csi::Keyboard::QueryFlags),
                    Csi::Device(csi::Device::RequestPrimaryDeviceAttributes)
                )?;
                handle.flush()?;

                loop {
                    let res = block_on(timeout(
                        Duration::from_millis(100),
                        params.events.query_events.recv(),
                    ));
                    let Ok(Some(res)) = res else {
                        break;
                    };
                    match res {
                        termina::Event::Csi(Csi::Keyboard(csi::Keyboard::ReportFlags(_))) => {
                            supports_keyboard_enhancement = true;
                        }
                        termina::Event::Csi(Csi::Device(csi::Device::DeviceAttributes(()))) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Ok::<_, io::Error>((profile, supports_keyboard_enhancement))
        })
        .await
        .unwrap()?;
        let handle = params.handle.clone();
        let mut termina_settings =
            rooibos_terminal::termina::TerminalSettings::from_writer(move || handle.clone())
                .raw_mode(false)
                .alternate_screen(settings.alternate_screen)
                .bracketed_paste(settings.bracketed_paste)
                .focus_change(settings.focus_change)
                .keyboard_enhancement(
                    settings.keyboard_enhancement && supports_keyboard_enhancement,
                )
                .force_keyboard_enhancement(supports_keyboard_enhancement)
                .mouse_capture(settings.mouse_capture);
        if let Some(title) = settings.title {
            termina_settings = termina_settings.title(title);
        }

        let inner = TerminaBackend::new(termina_settings).await;
        let window_size = params.events.window_size;

        Ok(Self {
            event_rx: Mutex::new(Some(params.events.events)),
            window_size,
            inner,
            profile,
        })
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

    fn window_size(
        &self,
        _backend: &mut Self::TuiBackend,
    ) -> io::Result<ratatui::backend::WindowSize> {
        Ok(*self.window_size.read().unwrap())
    }

    fn setup_terminal(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        self.inner.setup_terminal(&mut backend.inner)
    }

    fn restore_terminal(&self) -> io::Result<()> {
        let mut terminal = PlatformTerminal::new()?;

        self.inner.restore_terminal()?;
        terminal.enter_cooked_mode()
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

    fn set_clipboard<T: std::fmt::Display>(
        &self,
        backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: rooibos_terminal::ClipboardKind,
    ) -> io::Result<()> {
        self.inner
            .set_clipboard(&mut backend.inner, content, clipboard_kind)
    }

    fn color_palette(&self) -> ColorPalette {
        // TODO: support OSC 10/11 here
        ColorPalette::default()
    }

    fn profile(&self) -> tui_theme::profile::TermProfile {
        self.profile
    }

    fn async_input_stream(&self, cancellation_token: CancellationToken) -> impl AsyncInputStream {
        let event_rx = self.event_rx.lock().unwrap().take().unwrap();
        ReceiverStream::new(event_rx).take_until_if(async move {
            cancellation_token.cancelled().await;
            true
        })
    }
}

struct ForceTerminal;

impl IsTerminal for ForceTerminal {
    fn is_terminal(&self) -> bool {
        true
    }
}

pub struct SshTuiBackend {
    inner: tui::TerminaBackend<ArcHandle>,
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

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        self.inner.get_cursor_position()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        self.inner.set_cursor_position(position)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn size(&self) -> io::Result<Size> {
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
