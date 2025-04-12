use std::fmt::Display;
use std::io::{self, Read, Stderr, Stdout, Write, stderr, stdout};
use std::ops::Range;
use std::u16;

use ratatui::backend::{self, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Rect, Size};
use ratatui::style::{Color, Modifier};
use termina::escape::csi::{
    self, Csi, Cursor, DecPrivateMode, DecPrivateModeCode, Edit, KittyKeyboardFlags, Mode,
    SgrAttributes, SgrModifiers,
};
use termina::escape::dcs::Dcs;
use termina::escape::{CSI, OneBased, dcs, osc};
use termina::style::{ColorSpec, CursorStyle, RgbColor, RgbaColor, Stylized};
use termina::{Event, EventStream, PlatformTerminal, Terminal};
use terminput_termina::to_terminput;
use tokio_stream::StreamExt;
use tracing::debug;

use crate::{AsyncInputStream, AutoStream, Backend};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum KittyKeyboardSupport {
    /// The terminal doesn't support the protocol.
    #[default]
    None,
    /// The terminal supports the protocol but we haven't checked yet whether it has full or
    /// partial support for the flags we require.
    Some,
    /// The terminal only supports some of the flags we require.
    Partial,
    /// The terminal supports all flags require.
    Full,
}

#[derive(Debug, Default, Clone, Copy)]
struct Capabilities {
    kitty_keyboard: KittyKeyboardSupport,
    synchronized_output: bool,
    true_color: bool,
    extended_underlines: bool,
}

pub struct TerminaTuiBackend<W> {
    capabilities: Capabilities,
    terminal: PlatformTerminal,
    cursor_style: CursorStyle,
    writer: W,
}

impl<W> Write for TerminaTuiBackend<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W> TerminaTuiBackend<W>
where
    W: Write,
{
    fn new(writer: W) -> io::Result<Self> {
        let mut terminal = PlatformTerminal::new()?;
        let capabilities = Self::detect_capabilities(&mut terminal)?;
        Ok(Self {
            terminal,
            capabilities,
            writer,
            cursor_style: CursorStyle::Default,
        })
    }

    fn detect_capabilities(terminal: &mut PlatformTerminal) -> io::Result<Capabilities> {
        use std::time::Instant;

        // Colibri "midnight"
        const TEST_COLOR: RgbColor = RgbColor::new(59, 34, 76);

        terminal.enter_raw_mode()?;

        let mut capabilities = Capabilities::default();
        let start = Instant::now();

        // Many terminal extensions can be detected by querying the terminal for the state of the
        // extension and then sending a request for the primary device attributes (which is
        // consistently supported by all terminals). If we receive the status of the feature (for
        // example the current Kitty keyboard flags) then we know that the feature is supported.
        // If we only receive the device attributes then we know it is not.
        write!(
            terminal,
            "{}{}{}{}{}{}{}",
            // Kitty keyboard
            Csi::Keyboard(csi::Keyboard::QueryFlags),
            // Synchronized output
            Csi::Mode(csi::Mode::QueryDecPrivateMode(csi::DecPrivateMode::Code(
                csi::DecPrivateModeCode::SynchronizedOutput
            ))),
            // True color and while we're at it, extended underlines:
            // <https://github.com/termstandard/colors?tab=readme-ov-file#querying-the-terminal>
            Csi::Sgr(csi::Sgr::Background(TEST_COLOR.into())),
            Csi::Sgr(csi::Sgr::UnderlineColor(TEST_COLOR.into())),
            Dcs::Request(dcs::DcsRequest::GraphicRendition),
            Csi::Sgr(csi::Sgr::Reset),
            // Finally request the primary device attributes
            Csi::Device(csi::Device::RequestPrimaryDeviceAttributes),
        )?;
        terminal.flush()?;

        loop {
            match terminal.read(Event::is_escape)? {
                Event::Csi(Csi::Keyboard(csi::Keyboard::ReportFlags(_))) => {
                    capabilities.kitty_keyboard = KittyKeyboardSupport::Some;
                }
                Event::Csi(Csi::Mode(csi::Mode::ReportDecPrivateMode {
                    mode: csi::DecPrivateMode::Code(csi::DecPrivateModeCode::SynchronizedOutput),
                    setting: csi::DecModeSetting::Set | csi::DecModeSetting::Reset,
                })) => {
                    capabilities.synchronized_output = true;
                }
                Event::Dcs(dcs::Dcs::Response {
                    value: dcs::DcsResponse::GraphicRendition(sgrs),
                    ..
                }) => {
                    capabilities.true_color =
                        sgrs.contains(&csi::Sgr::Background(TEST_COLOR.into()));
                    capabilities.extended_underlines =
                        sgrs.contains(&csi::Sgr::UnderlineColor(TEST_COLOR.into()));
                }
                Event::Csi(Csi::Device(csi::Device::DeviceAttributes(_))) => break,
                _ => (),
            }
        }

        let end = Instant::now();
        debug!(
            "Detected terminal capabilities in {:?}: {capabilities:?}",
            end.duration_since(start)
        );

        terminal.enter_cooked_mode()?;

        Ok(capabilities)
    }
}

macro_rules! decset {
    ($mode:ident) => {
        Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::$mode,
        )))
    };
}
macro_rules! decreset {
    ($mode:ident) => {
        Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::$mode,
        )))
    };
}

impl<W> ratatui::backend::Backend for TerminaTuiBackend<W>
where
    W: Write,
{
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        if self.capabilities.synchronized_output {
            write!(self.writer, "{}", decset!(SynchronizedOutput))?;
        }

        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut underline_color = Color::Reset;
        let mut modifier = Modifier::empty();
        let mut last_pos: Option<Position> = None;
        for (x, y, cell) in content {
            // Move the cursor if the previous location was not (x - 1, y)
            if !matches!(last_pos, Some(p) if x == p.x + 1 && y == p.y) {
                write!(
                    self.writer,
                    "{}",
                    Csi::Cursor(csi::Cursor::Position {
                        col: OneBased::from_zero_based(x),
                        line: OneBased::from_zero_based(y),
                    })
                )?;
            }
            last_pos = Some(Position { x, y });

            let mut attributes = SgrAttributes::default();
            if cell.fg != fg {
                attributes.foreground = Some(to_colorspec(cell.fg));
                fg = cell.fg;
            }
            if cell.bg != bg {
                attributes.background = Some(to_colorspec(cell.bg));
                bg = cell.bg;
            }
            if cell.modifier != modifier {
                attributes.modifiers = diff_modifiers(modifier, cell.modifier);
                modifier = cell.modifier;
            }
            if self.capabilities.extended_underlines && cell.underline_color != underline_color {
                attributes.underline_color = Some(to_colorspec(cell.underline_color));
                underline_color = cell.underline_color;
            }
            // `attributes` will be empty if nothing changed between two cells. Empty
            // `SgrAttributes` behave the same as a `Sgr::Reset` rather than a 'no-op' though so
            // we should avoid writing them if they're empty.
            if !attributes.is_empty() {
                write!(
                    self.writer,
                    "{}",
                    Csi::Sgr(csi::Sgr::Attributes(attributes))
                )?;
            }

            write!(self.writer, "{}", &cell.symbol())?;
        }

        write!(self.writer, "{}", Csi::Sgr(csi::Sgr::Reset))?;

        if self.capabilities.synchronized_output {
            write!(self.writer, "{}", decreset!(SynchronizedOutput))?;
        }

        Ok(())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        write!(self.writer, "{}", decreset!(ShowCursor))?;
        Write::flush(self)
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}{}",
            decset!(ShowCursor),
            Csi::Cursor(csi::Cursor::CursorStyle(self.cursor_style)),
        )?;
        Write::flush(self)
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        write!(
            self.terminal,
            "{}",
            csi::Csi::Cursor(csi::Cursor::RequestActivePositionReport),
        )?;
        self.terminal.flush()?;
        let event = self.terminal.read(|event| {
            matches!(
                event,
                Event::Csi(Csi::Cursor(csi::Cursor::ActivePositionReport { .. }))
            )
        })?;
        let Event::Csi(Csi::Cursor(csi::Cursor::ActivePositionReport { line, col })) = event else {
            unreachable!();
        };
        Ok(Position {
            x: col.get_zero_based(),
            y: line.get_zero_based(),
        })
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> std::io::Result<()> {
        let position: Position = position.into();
        let col = OneBased::from_zero_based(position.x);
        let line = OneBased::from_zero_based(position.y);
        write!(
            self.writer,
            "{}",
            Csi::Cursor(csi::Cursor::Position { line, col })
        )?;
        Write::flush(self)
    }

    fn clear(&mut self) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}",
            Csi::Edit(csi::Edit::EraseInDisplay(csi::EraseInDisplay::EraseDisplay))
        )?;
        Write::flush(self)
    }

    fn size(&self) -> std::io::Result<Size> {
        let (rows, cols) = self.terminal.get_dimensions()?;
        Ok(Size::new(cols, rows))
    }

    fn window_size(&mut self) -> std::io::Result<WindowSize> {
        let size = self.size()?;
        Ok(WindowSize {
            columns_rows: size,
            pixels: Size::default(),
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    fn scroll_region_up(&mut self, region: Range<u16>, line_count: u16) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}{}{}",
            Csi::Cursor(Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(region.start),
                bottom: OneBased::from_zero_based(region.end)
            }),
            Csi::Edit(Edit::ScrollUp(line_count as u32)),
            Csi::Cursor(Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(0),
                bottom: OneBased::from_zero_based(u16::MAX - 1)
            })
        )
    }

    fn scroll_region_down(
        &mut self,
        region: std::ops::Range<u16>,
        line_count: u16,
    ) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}{}{}",
            Csi::Cursor(Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(region.start),
                bottom: OneBased::from_zero_based(region.end)
            }),
            Csi::Edit(Edit::ScrollDown(line_count as u32)),
            Csi::Cursor(Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(0),
                bottom: OneBased::from_zero_based(u16::MAX - 1)
            })
        )
    }
}

fn to_colorspec(color: Color) -> ColorSpec {
    if Stylized::is_ansi_color_disabled() {
        return ColorSpec::Reset;
    }
    match color {
        Color::Reset => ColorSpec::Reset,
        Color::Black => ColorSpec::BLACK,
        Color::Red => ColorSpec::RED,
        Color::Green => ColorSpec::GREEN,
        Color::Yellow => ColorSpec::YELLOW,
        Color::Blue => ColorSpec::BLUE,
        Color::Magenta => ColorSpec::MAGENTA,
        Color::Cyan => ColorSpec::CYAN,
        Color::Gray => ColorSpec::WHITE,
        Color::DarkGray => ColorSpec::BRIGHT_BLACK,
        Color::LightRed => ColorSpec::BRIGHT_RED,
        Color::LightGreen => ColorSpec::BRIGHT_GREEN,
        Color::LightYellow => ColorSpec::BRIGHT_YELLOW,
        Color::LightBlue => ColorSpec::BRIGHT_BLUE,
        Color::LightMagenta => ColorSpec::BRIGHT_MAGENTA,
        Color::LightCyan => ColorSpec::BRIGHT_CYAN,
        Color::White => ColorSpec::BRIGHT_WHITE,
        Color::Rgb(red, green, blue) => ColorSpec::TrueColor(RgbaColor {
            red,
            green,
            blue,
            alpha: 255,
        }),
        Color::Indexed(idx) => ColorSpec::PaletteIndex(idx),
    }
}

fn diff_modifiers(from: Modifier, to: Modifier) -> SgrModifiers {
    let mut modifiers = SgrModifiers::default();

    let removed = from - to;
    if removed.contains(Modifier::REVERSED) {
        modifiers |= SgrModifiers::NO_REVERSE;
    }
    if removed.contains(Modifier::BOLD) || removed.contains(Modifier::DIM) {
        modifiers |= SgrModifiers::INTENSITY_NORMAL;
    }
    if removed.contains(Modifier::ITALIC) {
        modifiers |= SgrModifiers::NO_ITALIC;
    }
    if removed.contains(Modifier::CROSSED_OUT) {
        modifiers |= SgrModifiers::NO_STRIKE_THROUGH;
    }
    if removed.contains(Modifier::HIDDEN) {
        modifiers |= SgrModifiers::NO_INVISIBLE;
    }
    if removed.contains(Modifier::SLOW_BLINK) || removed.contains(Modifier::RAPID_BLINK) {
        modifiers |= SgrModifiers::BLINK_NONE;
    }

    let added = to - from;
    if added.contains(Modifier::REVERSED) {
        modifiers |= SgrModifiers::REVERSE;
    }
    if added.contains(Modifier::BOLD) {
        modifiers |= SgrModifiers::INTENSITY_BOLD;
    }
    if added.contains(Modifier::DIM) {
        modifiers |= SgrModifiers::INTENSITY_DIM;
    }
    if added.contains(Modifier::ITALIC) {
        modifiers |= SgrModifiers::ITALIC;
    }
    if added.contains(Modifier::CROSSED_OUT) {
        modifiers |= SgrModifiers::STRIKE_THROUGH;
    }
    if added.contains(Modifier::HIDDEN) {
        modifiers |= SgrModifiers::INVISIBLE;
    }
    if added.contains(Modifier::SLOW_BLINK) {
        modifiers |= SgrModifiers::BLINK_SLOW;
    }
    if added.contains(Modifier::RAPID_BLINK) {
        modifiers |= SgrModifiers::BLINK_RAPID;
    }

    modifiers
}

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
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
    pub fn stdout() -> Self {
        adjust_color_output(&stdout());
        Self::from_writer(stdout)
    }
}

impl TerminalSettings<Stderr> {
    pub fn stderr() -> Self {
        adjust_color_output(&stderr());
        Self::from_writer(stderr)
    }
}

fn adjust_color_output<T>(writer: &T)
where
    T: io::IsTerminal + ?Sized,
{
    if let Some(enabled) = super::color_override() {
        Stylized::force_ansi_color(enabled);
    } else if !writer.is_terminal() {
        Stylized::force_ansi_color(false);
    }
}

impl TerminalSettings<AutoStream> {
    pub fn auto() -> Self {
        Self::from_writer(|| AutoStream::new(|term| adjust_color_output(term)))
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
    supports_keyboard_enhancement: bool,
}

impl<W: Write> TerminaBackend<W> {
    pub fn new(settings: TerminalSettings<W>) -> Self {
        let mut this = Self {
            settings,
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
        // if self.settings.keyboard_enhancement {
        //     let mut terminal = PlatformTerminal::new().unwrap();
        //     write!(
        //         terminal,
        //         "{}{}",
        //         Csi::Keyboard(csi::Keyboard::QueryFlags),
        //         Csi::Device(csi::Device::RequestPrimaryDeviceAttributes)
        //     )
        //     .unwrap();
        //     loop {
        //         match terminal.read(Event::is_escape).unwrap() {
        //             Event::Csi(Csi::Keyboard(csi::Keyboard::ReportFlags(_))) => {
        //                 self.supports_keyboard_enhancement = true;
        //             }
        //             Event::Csi(Csi::Device(csi::Device::DeviceAttributes(()))) => {
        //                 break;
        //             }
        //             _ => {}
        //         }
        //     }
        // }
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
    type TuiBackend = TerminaTuiBackend<W>;

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        let writer = (self.settings.get_writer)();
        TerminaTuiBackend::new(writer)
    }

    fn window_size(&self) -> io::Result<WindowSize> {
        let terminal = PlatformTerminal::new()?;
        let (rows, cols) = terminal.get_dimensions()?;
        Ok(WindowSize {
            columns_rows: Size {
                width: cols,
                height: rows,
            },
            pixels: Size::default(),
        })
    }

    fn setup_terminal(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        if self.settings.raw_mode {
            backend.terminal.enter_raw_mode()?;
        }
        let mut s = String::new();
        // hide cursor
        s += &format!("{CSI}?25l");
        if self.settings.alternate_screen {
            s += &decset!(ClearAndEnableAlternateScreen).to_string();
        }
        if self.settings.mouse_capture {
            s += &format!(
                "{}{}{}{}{}",
                decset!(MouseTracking),
                decset!(ButtonEventMouse),
                decset!(AnyEventMouse),
                decset!(RXVTMouse),
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
                KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS,
            ))
            .to_string();
        }
        if let Some(title) = &self.settings.title {
            s += &osc::Osc::SetWindowTitle(title).to_string();
        }
        write!(backend.terminal, "{}", s).unwrap();
        backend.terminal.flush()?;
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
                "{}{}{}{}{}",
                decreset!(MouseTracking),
                decreset!(ButtonEventMouse),
                decreset!(AnyEventMouse),
                decreset!(RXVTMouse),
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
        // show cursor;
        s += &format!("{CSI}?25h");
        write!(writer, "{}", s).unwrap();
        writer.flush()?;

        Ok(())
    }

    fn enter_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        write!(
            backend.terminal,
            "{}",
            decset!(ClearAndEnableAlternateScreen)
        )?;
        backend.terminal.flush()
    }

    fn leave_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        write!(
            backend.terminal,
            "{}",
            decreset!(ClearAndEnableAlternateScreen)
        )?;
        backend.terminal.flush()
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        backend: &mut Self::TuiBackend,
        title: T,
    ) -> io::Result<()> {
        write!(
            backend.terminal,
            "{}",
            osc::Osc::SetWindowTitle(&title.to_string())
        )?;
        backend.terminal.flush()
    }

    fn set_clipboard<T: Display>(
        &self,
        backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: super::ClipboardKind,
    ) -> io::Result<()> {
        // #[cfg(feature = "clipboard")]
        // return execute!(
        //     backend,
        //     SetClipboard::new(&content.to_string(), clipboard_kind)
        // );
        #[cfg(not(feature = "clipboard"))]
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "clipboard feature not enabled",
        ));
        Ok(())
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.supports_keyboard_enhancement
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        (self.settings.get_writer)().write_all(buf)
    }

    fn async_input_stream(&self) -> impl AsyncInputStream {
        let event_reader = PlatformTerminal::new().unwrap().event_stream(|_| true);

        event_reader.filter_map(move |e| {
            if let Ok(e) = e {
                let e = to_terminput(e);
                return e.ok();
            }
            None
        })
    }
}
