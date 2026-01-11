use std::io::{self, Write};

use ratatui::backend::{ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use ratatui::style::{Color, Modifier};
use termina::escape::csi::{self, Csi, SgrAttributes, SgrModifiers};
use termina::style::{ColorSpec, RgbColor, RgbaColor};
use termina::{Event, OneBased, PlatformTerminal, Terminal};

use crate::termina::macros::{decreset, decset};

#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    pub(crate) synchronized_output: bool,
}

#[derive(Debug)]
pub struct TerminaBackend<W: Write> {
    terminal: PlatformTerminal,
    writer: W,
    capabilities: Capabilities,
    is_synchronized_output_set: bool,
}

impl<W> TerminaBackend<W>
where
    W: Write,
{
    pub const fn new(terminal: PlatformTerminal, capabilities: Capabilities, writer: W) -> Self {
        Self {
            terminal,
            capabilities,
            writer,
            is_synchronized_output_set: false,
        }
    }

    pub const fn writer(&self) -> &W {
        &self.writer
    }

    pub const fn terminal(&self) -> &PlatformTerminal {
        &self.terminal
    }

    pub const fn terminal_mut(&mut self) -> &mut PlatformTerminal {
        &mut self.terminal
    }

    pub const fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    fn erase_in_display(&mut self, erase_in_display: csi::EraseInDisplay) -> io::Result<()> {
        write!(
            self.writer,
            "{}",
            Csi::Edit(csi::Edit::EraseInDisplay(erase_in_display))
        )?;
        self.writer.flush()
    }

    fn erase_in_line(&mut self, erase_in_line: csi::EraseInLine) -> io::Result<()> {
        write!(
            self.writer,
            "{}",
            Csi::Edit(csi::Edit::EraseInLine(erase_in_line))
        )?;
        self.writer.flush()
    }

    fn start_synchronized_render(&mut self) -> io::Result<()> {
        if self.capabilities.synchronized_output && !self.is_synchronized_output_set {
            write!(self.writer, "{}", decset!(SynchronizedOutput))?;
            self.writer.flush()?;
            self.is_synchronized_output_set = true;
        }
        Ok(())
    }

    fn end_sychronized_render(&mut self) -> io::Result<()> {
        if self.is_synchronized_output_set {
            write!(self.writer, "{}", decreset!(SynchronizedOutput))?;
            self.writer.flush()?;
            self.is_synchronized_output_set = false;
        }
        Ok(())
    }
}

impl<W> Write for TerminaBackend<W>
where
    W: Write,
{
    /// Writes a buffer of bytes to the underlying buffer.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    /// Flushes the underlying buffer.
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W> ratatui::backend::Backend for TerminaBackend<W>
where
    W: Write,
{
    type Error = io::Error;

    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        self.start_synchronized_render()?;

        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        // let mut underline_color = Color::Reset;
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
                attributes.foreground = Some(cell.fg.into_termina());
                fg = cell.fg;
            }
            if cell.bg != bg {
                attributes.background = Some(cell.bg.into_termina());
                bg = cell.bg;
            }
            if cell.modifier != modifier {
                attributes.modifiers = diff_modifiers(modifier, cell.modifier);
                modifier = cell.modifier;
            }
            // if cell.underline_color != underline_color {
            //     write!(
            //         self.writer,
            //         "{}",
            //         Csi::Sgr(csi::Sgr::UnderlineColor(
            //             cell.underline_color.into_termina()
            //         ))
            //     )?;
            //     underline_color = cell.underline_color;
            // }

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
        self.end_sychronized_render()?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.writer, "{}", decreset!(ShowCursor))?;
        self.writer.flush()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.writer, "{}", decset!(ShowCursor))?;
        self.writer.flush()
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

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        let position: Position = position.into();
        let col = OneBased::from_zero_based(position.x);
        let line = OneBased::from_zero_based(position.y);
        write!(
            self.writer,
            "{}",
            Csi::Cursor(csi::Cursor::Position { line, col })
        )?;
        self.writer.flush()
    }

    fn clear(&mut self) -> io::Result<()> {
        self.clear_region(ClearType::All)
    }

    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        match clear_type {
            ClearType::All => self.erase_in_display(csi::EraseInDisplay::EraseDisplay),
            ClearType::AfterCursor => {
                self.erase_in_display(csi::EraseInDisplay::EraseToEndOfDisplay)
            }
            ClearType::BeforeCursor => {
                self.erase_in_display(csi::EraseInDisplay::EraseToStartOfDisplay)
            }
            ClearType::CurrentLine => self.erase_in_line(csi::EraseInLine::EraseLine),
            ClearType::UntilNewLine => self.erase_in_line(csi::EraseInLine::EraseToEndOfLine),
        }
    }

    fn append_lines(&mut self, n: u16) -> io::Result<()> {
        for _ in 0..n {
            writeln!(self.writer)?;
        }
        self.writer.flush()
    }

    fn size(&self) -> io::Result<Size> {
        let termina::WindowSize { rows, cols, .. } = self.terminal.get_dimensions()?;
        Ok(Size {
            width: cols,
            height: rows,
        })
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        let termina::WindowSize {
            rows,
            cols,
            pixel_width,
            pixel_height,
        } = self.terminal.get_dimensions()?;
        Ok(WindowSize {
            columns_rows: Size {
                width: cols,
                height: rows,
            },
            pixels: Size {
                width: pixel_width.unwrap_or_default(),
                height: pixel_height.unwrap_or_default(),
            },
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_up(&mut self, region: std::ops::Range<u16>, amount: u16) -> io::Result<()> {
        if amount == 0 {
            return Ok(());
        }
        write!(
            self.writer,
            "{}{}{}",
            Csi::Cursor(csi::Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(region.start),
                bottom: OneBased::from_zero_based(region.end.saturating_sub(1))
            }),
            Csi::Edit(csi::Edit::ScrollUp(u32::from(amount))),
            Csi::Cursor(csi::Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(0),
                bottom: OneBased::from_zero_based(u16::MAX - 1)
            })
        )?;
        self.writer.flush()
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_down(&mut self, region: std::ops::Range<u16>, amount: u16) -> io::Result<()> {
        if amount == 0 {
            return Ok(());
        }
        write!(
            self.writer,
            "{}{}{}",
            Csi::Cursor(csi::Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(region.start),
                bottom: OneBased::from_zero_based(region.end.saturating_sub(1))
            }),
            Csi::Edit(csi::Edit::ScrollDown(u32::from(amount))),
            Csi::Cursor(csi::Cursor::SetTopAndBottomMargins {
                top: OneBased::from_zero_based(0),
                bottom: OneBased::from_zero_based(u16::MAX - 1)
            })
        )?;
        self.writer.flush()
    }
}

/// A trait for converting a Ratatui type to a termina type.
///
/// This trait is needed for avoiding the orphan rule when implementing `From` for termina types
/// once these are moved to a separate crate.
pub trait IntoTermina<C> {
    /// Converts the ratatui type to a termina type.
    fn into_termina(self) -> C;
}

/// A trait for converting a termina type to a Ratatui type.
///
/// This trait is needed for avoiding the orphan rule when implementing `From` for termina types
/// once these are moved to a separate crate.
pub trait FromTermina<C> {
    /// Converts the termina type to a ratatui type.
    fn from_termina(value: C) -> Self;
}

impl IntoTermina<ColorSpec> for Color {
    fn into_termina(self) -> ColorSpec {
        match self {
            Self::Reset => ColorSpec::Reset,
            Self::Black => ColorSpec::BLACK,
            Self::Red => ColorSpec::RED,
            Self::Green => ColorSpec::GREEN,
            Self::Yellow => ColorSpec::YELLOW,
            Self::Blue => ColorSpec::BLUE,
            Self::Magenta => ColorSpec::MAGENTA,
            Self::Cyan => ColorSpec::CYAN,
            Self::Gray => ColorSpec::WHITE,
            Self::DarkGray => ColorSpec::BRIGHT_BLACK,
            Self::LightRed => ColorSpec::BRIGHT_RED,
            Self::LightGreen => ColorSpec::BRIGHT_GREEN,
            Self::LightBlue => ColorSpec::BRIGHT_BLUE,
            Self::LightYellow => ColorSpec::BRIGHT_YELLOW,
            Self::LightMagenta => ColorSpec::BRIGHT_MAGENTA,
            Self::LightCyan => ColorSpec::BRIGHT_CYAN,
            Self::White => ColorSpec::BRIGHT_WHITE,
            Self::Indexed(i) => ColorSpec::PaletteIndex(i),
            Self::Rgb(r, g, b) => ColorSpec::TrueColor(
                RgbColor {
                    red: r,
                    green: g,
                    blue: b,
                }
                .into(),
            ),
        }
    }
}

impl FromTermina<ColorSpec> for Color {
    fn from_termina(value: ColorSpec) -> Self {
        match value {
            ColorSpec::Reset => Self::Reset,
            ColorSpec::BLACK => Self::Black,
            ColorSpec::RED => Self::Red,
            ColorSpec::GREEN => Self::Green,
            ColorSpec::YELLOW => Self::Yellow,
            ColorSpec::BLUE => Self::Blue,
            ColorSpec::MAGENTA => Self::Magenta,
            ColorSpec::CYAN => Self::Cyan,
            ColorSpec::WHITE => Self::Gray,
            ColorSpec::BRIGHT_BLACK => Self::DarkGray,
            ColorSpec::BRIGHT_RED => Self::LightRed,
            ColorSpec::BRIGHT_GREEN => Self::LightGreen,
            ColorSpec::BRIGHT_BLUE => Self::LightBlue,
            ColorSpec::BRIGHT_YELLOW => Self::LightYellow,
            ColorSpec::BRIGHT_MAGENTA => Self::LightMagenta,
            ColorSpec::BRIGHT_CYAN => Self::LightCyan,
            ColorSpec::BRIGHT_WHITE => Self::White,
            ColorSpec::TrueColor(RgbaColor {
                red, green, blue, ..
            }) => Self::Rgb(red, green, blue),
            ColorSpec::PaletteIndex(v) => Self::Indexed(v),
        }
    }
}

fn diff_modifiers(from: Modifier, to: Modifier) -> SgrModifiers {
    let mut modifiers = SgrModifiers::default();

    let removed = from - to;
    if removed.contains(Modifier::REVERSED) {
        modifiers |= SgrModifiers::NO_REVERSE;
    }
    if removed.contains(Modifier::BOLD) && !to.contains(Modifier::DIM) {
        modifiers |= SgrModifiers::INTENSITY_NORMAL;
    }
    if removed.contains(Modifier::DIM) {
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
