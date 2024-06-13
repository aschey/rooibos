use std::io::{self, stderr, stdout, Stderr, Stdout, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::{DisableBlinking, Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    KeyboardEnhancementFlags, MouseEvent, MouseEventKind, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen, SetTitle,
};
use crossterm::{execute, queue};
use futures::StreamExt;
use ratatui::{Terminal, Viewport};
use ratatui_wasm::xterm::Theme;
use ratatui_wasm::{init_terminal, EventStream, TerminalHandle};
use rooibos_runtime::backend::Backend;
use rooibos_runtime::{wasm_compat, SignalMode};
use tap::TapFallible;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;
use web_sys::wasm_bindgen::JsCast;

pub struct TerminalSettings {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    focus_change: bool,
    bracketed_paste: bool,
    viewport: Viewport,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            focus_change: true,
            bracketed_paste: true,
            viewport: Viewport::default(),
        }
    }
}

impl TerminalSettings {
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
}

pub struct WasmBackend {
    settings: TerminalSettings,
    supports_keyboard_enhancement: bool,
}

impl WasmBackend {
    pub fn new(settings: TerminalSettings) -> Self {
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

impl Default for WasmBackend {
    fn default() -> Self {
        Self::new(TerminalSettings::default())
    }
}

impl Backend for WasmBackend {
    type TuiBackend = ratatui_wasm::CrosstermBackend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>> {
        let elem = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("terminal")
            .unwrap();

        init_terminal(
            ratatui_wasm::xterm::TerminalOptions::new()
                .with_rows(50)
                .with_cursor_blink(true)
                .with_cursor_width(10)
                .with_font_size(20)
                .with_draw_bold_text_in_bright_colors(true)
                .with_right_click_selects_word(true)
                .with_theme(
                    Theme::new()
                        .with_foreground("#98FB98")
                        .with_background("#000000"),
                ),
            elem.dyn_into().unwrap(),
        );

        let mut handle = TerminalHandle::default();

        queue!(handle, Hide)?;
        if self.settings.alternate_screen {
            queue!(handle, EnterAlternateScreen)?;
        }
        if self.settings.mouse_capture {
            queue!(handle, EnableMouseCapture)?;
        }
        if self.settings.focus_change {
            queue!(handle, EnableFocusChange)?;
        }
        if self.settings.bracketed_paste {
            queue!(handle, EnableBracketedPaste)?;
        }

        if self.supports_keyboard_enhancement {
            queue!(
                handle,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
            )?;
        }
        handle.flush()?;

        let mut terminal = Terminal::with_options(
            ratatui_wasm::CrosstermBackend::new(handle),
            ratatui::TerminalOptions {
                viewport: self.settings.viewport.clone(),
            },
        )?;

        terminal.clear()?;
        Ok(terminal)
    }

    fn restore_terminal(&self) -> io::Result<()> {
        let mut handle = TerminalHandle::default();
        queue!(handle, DisableBlinking)?;
        if self.supports_keyboard_enhancement {
            queue!(handle, PopKeyboardEnhancementFlags)?;
        }
        if self.settings.mouse_capture {
            queue!(handle, DisableMouseCapture)?;
        }
        if self.settings.focus_change {
            queue!(handle, DisableFocusChange)?;
        }
        if self.settings.bracketed_paste {
            queue!(handle, DisableBracketedPaste)?;
        }
        if self.settings.alternate_screen {
            queue!(handle, LeaveAlternateScreen)?;
        }
        queue!(handle, Show)?;
        handle.flush()?;

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

    fn supports_keyboard_enhancement(&self) -> bool {
        self.supports_keyboard_enhancement
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        let mut handle = TerminalHandle::default();
        handle.write_all(buf)
    }

    async fn read_input(&self, term_tx: broadcast::Sender<rooibos_dom::Event>) {
        let mut event_reader = EventStream::new().fuse();
        while let Some(Ok(event)) = event_reader.next().await {
            if let Ok(event) = event.try_into() {
                let _ = term_tx
                    .send(event)
                    .tap_err(|e| warn!("failed to send event {e:?}"));
            }
        }
    }
}
