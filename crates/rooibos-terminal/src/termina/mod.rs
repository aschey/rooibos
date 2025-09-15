pub mod tui;
use std::fmt::Display;
use std::io::{self, Stderr, Stdout, Write, stderr, stdout};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use ratatui::backend::WindowSize;
use ratatui::layout::Size;
use termina::escape::csi::{self, Csi, KittyKeyboardFlags};
use termina::escape::osc::{self, Selection};
use termina::{EventReader, PlatformTerminal, Terminal};
use terminput_termina::to_terminput;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tui_theme::profile::DetectorSettings;
use tui_theme::{color_palette, term_profile};

use super::Backend;
use crate::termina::macros::{decreset, decset};
use crate::{AsyncInputStream, AutoStream, StreamImpl};

pub(super) mod macros {
    macro_rules! decset {
        ($mode:ident) => {
            termina::escape::csi::Csi::Mode(termina::escape::csi::Mode::SetDecPrivateMode(
                termina::escape::csi::DecPrivateMode::Code(
                    termina::escape::csi::DecPrivateModeCode::$mode,
                ),
            ))
        };
    }
    macro_rules! decreset {
        ($mode:ident) => {
            termina::escape::csi::Csi::Mode(termina::escape::csi::Mode::ResetDecPrivateMode(
                termina::escape::csi::DecPrivateMode::Code(
                    termina::escape::csi::DecPrivateModeCode::$mode,
                ),
            ))
        };
    }

    pub(super) use {decreset, decset};
}

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    force_keyboard_enhancement: bool,
    focus_change: bool,
    bracketed_paste: bool,
    raw_mode: bool,
    title: Option<String>,
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self::stdout()
    }
}

impl Default for TerminalSettings<Stderr> {
    fn default() -> Self {
        Self::stderr()
    }
}

impl Default for TerminalSettings<AutoStream> {
    fn default() -> Self {
        Self::auto()
    }
}

impl TerminalSettings<Stdout> {
    pub fn stdout_with_detector_options(settings: DetectorSettings) -> Self {
        tui_theme::load_profile(&stdout(), settings);
        tui_theme::load_color_palette();
        Self::from_writer(stdout)
    }

    pub fn stdout() -> Self {
        Self::stdout_with_detector_options(DetectorSettings::new())
    }
}

impl TerminalSettings<Stderr> {
    pub fn stderr_with_detector_options(settings: DetectorSettings) -> Self {
        tui_theme::load_profile(&stderr(), settings);
        tui_theme::load_color_palette();
        Self::from_writer(stderr)
    }

    pub fn stderr() -> Self {
        Self::stderr_with_detector_options(DetectorSettings::new())
    }
}

impl TerminalSettings<AutoStream> {
    pub fn auto_with_detector_options(settings: DetectorSettings) -> Self {
        match AutoStream::new().0 {
            StreamImpl::Stdout(out) => tui_theme::load_profile(&out, settings),
            StreamImpl::Stderr(err) => tui_theme::load_profile(&err, settings),
        }
        tui_theme::load_color_palette();
        Self::from_writer(AutoStream::new)
    }
    pub fn auto() -> Self {
        Self::auto_with_detector_options(DetectorSettings::new())
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
            force_keyboard_enhancement: false,
            focus_change: true,
            bracketed_paste: true,
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

    pub fn keyboard_enhancement(mut self, keyboard_enhancement: bool) -> Self {
        self.keyboard_enhancement = keyboard_enhancement;
        self
    }

    pub fn force_keyboard_enhancement(mut self, force_keyboard_enhancement: bool) -> Self {
        self.force_keyboard_enhancement = force_keyboard_enhancement;
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

pub struct TerminaBackend<W: Write> {
    settings: TerminalSettings<W>,
    event_reader: Mutex<Option<EventReader>>,
    supports_keyboard_enhancement: bool,
}

impl<W: Write> TerminaBackend<W> {
    pub fn new(settings: TerminalSettings<W>) -> Self {
        let mut this = Self {
            settings,
            event_reader: None.into(),
            supports_keyboard_enhancement: false,
        };
        this.set_keyboard_enhancement();
        this
    }

    pub fn settings(mut self, settings: TerminalSettings<W>) -> Self {
        self.settings = settings;
        self.set_keyboard_enhancement();
        self
    }

    fn set_keyboard_enhancement(&mut self) {
        if self.settings.force_keyboard_enhancement {
            self.supports_keyboard_enhancement = true;
            return;
        }

        if !self.settings.raw_mode {
            return;
        }

        if self.settings.keyboard_enhancement {
            let mut terminal = PlatformTerminal::new().unwrap();
            terminal.enter_raw_mode().unwrap();
            write!(
                terminal,
                "{}{}",
                Csi::Keyboard(csi::Keyboard::QueryFlags),
                Csi::Device(csi::Device::RequestPrimaryDeviceAttributes)
            )
            .unwrap();
            terminal.flush().unwrap();
            loop {
                if !matches!(
                    terminal.poll(termina::Event::is_escape, Duration::from_millis(100).into()),
                    Ok(true)
                ) {
                    break;
                }
                match terminal.read(termina::Event::is_escape).unwrap() {
                    termina::Event::Csi(Csi::Keyboard(csi::Keyboard::ReportFlags(_))) => {
                        self.supports_keyboard_enhancement = true;
                    }
                    termina::Event::Csi(Csi::Device(csi::Device::DeviceAttributes(()))) => {
                        break;
                    }
                    _ => {}
                }
            }
            terminal.enter_cooked_mode().unwrap();
        }
    }
}

impl Default for TerminaBackend<Stdout> {
    fn default() -> Self {
        Self::new(TerminalSettings::default())
    }
}

impl Default for TerminaBackend<Stderr> {
    fn default() -> Self {
        Self::new(TerminalSettings::default())
    }
}

impl Default for TerminaBackend<AutoStream> {
    fn default() -> Self {
        Self::new(TerminalSettings::default())
    }
}

impl TerminaBackend<AutoStream> {
    pub fn auto() -> Self {
        Self::default()
    }
}

impl TerminaBackend<Stdout> {
    pub fn stdout() -> Self {
        Self::default()
    }
}

impl TerminaBackend<Stderr> {
    pub fn stderr() -> Self {
        Self::default()
    }
}

impl<W: Write> Backend for TerminaBackend<W> {
    type TuiBackend = tui::TerminaBackend<W>;

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        let writer = (self.settings.get_writer)();
        let mut terminal = PlatformTerminal::new()?;
        if self.settings.raw_mode {
            terminal.enter_raw_mode()?;
            *self.event_reader.lock().unwrap() = terminal.event_reader().into();
        }

        Ok(tui::TerminaBackend::new(terminal, writer))
    }

    fn window_size(&self, backend: &mut Self::TuiBackend) -> io::Result<WindowSize> {
        let size = backend.terminal().get_dimensions()?;
        Ok(WindowSize {
            columns_rows: Size {
                width: size.cols,
                height: size.rows,
            },
            pixels: Size {
                width: size.pixel_width.unwrap_or(0),
                height: size.pixel_height.unwrap_or(0),
            },
        })
    }

    fn setup_terminal(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        let mut s = String::new();
        let mut writer = (self.settings.get_writer)();
        s += &format!("{}", decreset!(ShowCursor));
        if self.settings.alternate_screen {
            s += &decset!(ClearAndEnableAlternateScreen).to_string();
        }
        if self.settings.mouse_capture {
            s += &format!(
                "{}{}{}{}",
                decset!(MouseTracking),
                decset!(ButtonEventMouse),
                decset!(AnyEventMouse),
                decset!(SGRMouse)
            );
        }
        if self.settings.focus_change {
            s += &decset!(FocusTracking).to_string();
        }
        if self.settings.bracketed_paste {
            s += &decset!(BracketedPaste).to_string();
        }
        if self.supports_keyboard_enhancement {
            s += &csi::Csi::Keyboard(csi::Keyboard::PushFlags(
                KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES | KittyKeyboardFlags::all(),
            ))
            .to_string();
        }
        if let Some(title) = &self.settings.title {
            s += &osc::Osc::SetWindowTitle(title).to_string();
        }

        write!(writer, "{}", s).unwrap();
        writer.flush()?;
        Ok(())
    }

    fn restore_terminal(&self) -> io::Result<()> {
        let mut terminal = PlatformTerminal::new()?;
        let mut writer = (self.settings.get_writer)();
        if self.settings.raw_mode {
            terminal.enter_cooked_mode()?;
        }
        let mut s = String::new();

        if self.supports_keyboard_enhancement {
            s += &csi::Csi::Keyboard(csi::Keyboard::PopFlags(1)).to_string();
        }
        if self.settings.mouse_capture {
            s += &format!(
                "{}{}{}{}",
                decreset!(MouseTracking),
                decreset!(ButtonEventMouse),
                decreset!(AnyEventMouse),
                decreset!(SGRMouse)
            );
        }
        if self.settings.focus_change {
            s += &decreset!(FocusTracking).to_string();
        }
        if self.settings.bracketed_paste {
            s += &decreset!(BracketedPaste).to_string();
        }
        if self.settings.alternate_screen {
            s += &decreset!(ClearAndEnableAlternateScreen).to_string();
        }
        s += &decset!(ShowCursor).to_string();

        write!(writer, "{s}")?;
        writer.flush()?;

        Ok(())
    }

    fn enter_alt_screen(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        let mut writer = (self.settings.get_writer)();
        write!(writer, "{}", decset!(ClearAndEnableAlternateScreen))?;
        writer.flush()
    }

    fn leave_alt_screen(&self, _backend: &mut Self::TuiBackend) -> io::Result<()> {
        let mut writer = (self.settings.get_writer)();
        write!(writer, "{}", decreset!(ClearAndEnableAlternateScreen))?;
        writer.flush()
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        _backend: &mut Self::TuiBackend,
        title: T,
    ) -> io::Result<()> {
        let mut writer = (self.settings.get_writer)();

        write!(writer, "{}", osc::Osc::SetWindowTitle(&title.to_string()))?;
        writer.flush()
    }

    fn set_clipboard<T: Display>(
        &self,
        _backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: super::ClipboardKind,
    ) -> io::Result<()> {
        let kind = match clipboard_kind {
            super::ClipboardKind::Primary => Selection::PRIMARY,
            super::ClipboardKind::Clipboard => Selection::CLIPBOARD,
        };
        let mut writer = (self.settings.get_writer)();

        write!(
            writer,
            "{}",
            osc::Osc::SetSelection(kind, &content.to_string())
        )?;
        writer.flush()
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.supports_keyboard_enhancement
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        (self.settings.get_writer)().write_all(buf)
    }

    fn color_palette(&self) -> tui_theme::ColorPalette {
        color_palette()
    }

    fn profile(&self) -> tui_theme::profile::TermProfile {
        term_profile()
    }

    fn async_input_stream(&self, cancellation_token: CancellationToken) -> impl AsyncInputStream {
        let (tx, rx) = mpsc::channel(1024);
        let Some(reader) = self.event_reader.lock().unwrap().take() else {
            return ReceiverStream::new(rx);
        };

        thread::spawn(move || {
            loop {
                let poll = reader.poll(Duration::from_millis(20).into(), |e| !e.is_escape());
                if cancellation_token.is_cancelled() {
                    while matches!(
                        reader.poll(Duration::from_millis(20).into(), |_| true),
                        Ok(true)
                    ) {
                        reader.read(|_| true).unwrap();
                    }
                    return;
                }
                if matches!(poll, Ok(true))
                    && let Ok(event) = reader.read(|e| !e.is_escape())
                    && let Ok(event) = to_terminput(event)
                {
                    let _ = tx.try_send(event);
                }
            }
        });
        ReceiverStream::new(rx)
    }
}
