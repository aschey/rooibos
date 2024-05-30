use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::{DisableBlinking, Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent, KeyModifiers,
    KeyboardEnhancementFlags, MouseEvent, MouseEventKind, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use futures_util::StreamExt;
use ratatui::{Terminal, Viewport};
use tap::TapFallible;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

use super::Backend;
use crate::SignalMode;

pub struct TerminalSettings<W> {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    focus_change: bool,
    bracketed_paste: bool,
    viewport: Viewport,
    hover_debounce: Duration,
    get_writer: Box<dyn Fn() -> W + Send + Sync>,
}

impl Default for TerminalSettings<Stdout> {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            focus_change: true,
            bracketed_paste: true,
            viewport: Viewport::default(),
            hover_debounce: Duration::from_millis(20),
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
            focus_change: true,
            bracketed_paste: true,
            viewport: Viewport::default(),
            hover_debounce: Duration::from_millis(20),
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

    pub fn hover_debounce(mut self, hover_debounce: Duration) -> Self {
        self.hover_debounce = hover_debounce;
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
        queue!(writer, DisableBlinking)?;
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

    fn enter_alt_screen(&self) -> io::Result<()> {
        let mut writer = (self.settings.get_writer)();
        execute!(writer, EnterAlternateScreen)
    }

    fn leave_alt_screen(&self) -> io::Result<()> {
        let mut writer = (self.settings.get_writer)();
        execute!(writer, LeaveAlternateScreen)
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.supports_keyboard_enhancement
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        (self.settings.get_writer)().write_all(buf)
    }

    async fn read_input(
        &self,
        signal_tx: mpsc::Sender<SignalMode>,
        term_tx: broadcast::Sender<rooibos_dom::Event>,
    ) {
        let hover_debounce = self.settings.hover_debounce.as_millis();
        let mut event_reader = EventStream::new().fuse();

        let mut last_move_time = Instant::now();
        let mut pending_move = None;
        loop {
            let send_next_move = tokio::time::sleep(Duration::from_millis(
                hover_debounce.saturating_sub((Instant::now() - last_move_time).as_millis()) as u64,
            ));

            tokio::select! {
                next_event = event_reader.next() => {
                    match next_event {
                        Some(Ok(
                            event @ Event::Key(KeyEvent {
                                code, modifiers, ..
                            }),
                        )) => {
                            if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
                                let _ = signal_tx
                                    .send(SignalMode::Terminate)
                                    .await
                                    .tap_err(|_| warn!("error sending terminate signal"));
                            } else if cfg!(unix) && modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('z')
                            {
                                let _ = signal_tx
                                    .send(SignalMode::Suspend)
                                    .await
                                    .tap_err(|_| warn!("error sending suspend signal"));
                            } else {
                                let _ = term_tx
                                    .send(event.into())
                                    .tap_err(|_| warn!("error sending terminal event"));
                            }
                        }
                        Some(Ok(
                            mouse_event @ Event::Mouse(MouseEvent {
                                kind: MouseEventKind::Moved,
                                ..
                            }),
                        )) => {
                            pending_move = Some(mouse_event);
                            last_move_time = Instant::now();
                        }
                        Some(Ok(event)) => {
                            term_tx.send(event.into()).ok();
                        }
                        _ => {
                            return;
                        }
                    }
                }
                _ = send_next_move, if pending_move.is_some() => {
                    term_tx.send(pending_move.take().unwrap().into()).ok();
                }
            }
        }
    }
}
