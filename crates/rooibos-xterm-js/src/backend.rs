use std::fmt::Display;
use std::io::{self, Write};

use crossterm::cursor::{DisableBlinking, Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, SetTitle, supports_keyboard_enhancement,
};
use crossterm::{execute, queue};
use futures::Future;
use ratatui::backend::WindowSize;
use ratatui::layout::Size;
use ratatui::{Terminal, Viewport};
use ratatui_xterm_js::xterm::Theme;
use ratatui_xterm_js::{EventStream, TerminalHandle, XtermJsBackend, init_terminal};
use rooibos_terminal::{AsyncInputStream, Backend, ClipboardKind};
use tap::TapFallible;
use terminput_crossterm::to_terminput;
use tokio::sync::broadcast;
use tokio_stream::StreamExt as _;
use tracing::warn;
use web_sys::wasm_bindgen::JsCast;

pub struct TerminalSettings {
    alternate_screen: bool,
    mouse_capture: bool,
    keyboard_enhancement: bool,
    focus_change: bool,
    bracketed_paste: bool,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            keyboard_enhancement: true,
            focus_change: true,
            bracketed_paste: true,
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
        let elem = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("terminal")
            .unwrap();

        init_terminal(
            ratatui_xterm_js::xterm::TerminalOptions::new()
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
    type TuiBackend = XtermJsBackend;

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend> {
        let mut handle = TerminalHandle::default();
        Ok(XtermJsBackend::new(handle))
    }

    fn window_size(&self) -> io::Result<WindowSize> {
        let crossterm::terminal::WindowSize {
            columns,
            rows,
            width,
            height,
        } = ratatui_xterm_js::window_size()?;
        Ok(WindowSize {
            columns_rows: Size {
                width: columns,
                height: rows,
            },
            pixels: Size { width, height },
        })
    }

    fn setup_terminal(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        queue!(backend, Hide)?;
        if self.settings.alternate_screen {
            queue!(backend, EnterAlternateScreen)?;
        }
        if self.settings.mouse_capture {
            queue!(backend, EnableMouseCapture)?;
        }
        if self.settings.focus_change {
            queue!(backend, EnableFocusChange)?;
        }
        if self.settings.bracketed_paste {
            queue!(backend, EnableBracketedPaste)?;
        }

        if self.supports_keyboard_enhancement {
            queue!(
                backend,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
            )?;
        }
        backend.flush()?;
        Ok(())
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

    fn enter_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        execute!(backend, EnterAlternateScreen)
    }

    fn leave_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()> {
        execute!(backend, LeaveAlternateScreen)
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        backend: &mut Self::TuiBackend,
        title: T,
    ) -> io::Result<()> {
        execute!(backend, SetTitle(title))
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        self.supports_keyboard_enhancement
    }

    fn set_clipboard<T: Display>(
        &self,
        backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: ClipboardKind,
    ) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        let mut handle = TerminalHandle::default();
        handle.write_all(buf)
    }

    fn async_input_stream(&self) -> impl AsyncInputStream {
        let event_reader = EventStream::new().fuse();
        event_reader.filter_map(move |e| {
            if let Ok(e) = e {
                let e: Result<rooibos_dom::Event, _> = to_terminput(e);
                return e.ok();
            }
            None
        })
    }
}
