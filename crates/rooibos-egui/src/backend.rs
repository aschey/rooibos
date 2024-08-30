use std::fmt::Display;
use std::io;
use std::sync::Mutex;

use egui::InputState;
use egui_ratatui::RataguiBackend;
use futures_cancel::FutureExt;
use ratatui::Terminal;
use rooibos_dom::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent, MouseEventKind,
};
use rooibos_runtime::backend::Backend;
use rooibos_runtime::ServiceContext;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

pub struct TerminalSettings {}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalSettings {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct EguiBackend {
    settings: TerminalSettings,
    event_rx: Mutex<Option<mpsc::Receiver<Vec<egui::Event>>>>,
    width: u16,
    height: u16,
    font_size: u16,
}

impl EguiBackend {
    pub fn new(width: u16, height: u16, event_rx: mpsc::Receiver<Vec<egui::Event>>) -> Self {
        Self::new_with_settings(width, height, event_rx, TerminalSettings::default())
    }

    pub fn new_with_settings(
        width: u16,
        height: u16,
        event_rx: mpsc::Receiver<Vec<egui::Event>>,
        settings: TerminalSettings,
    ) -> Self {
        Self {
            settings,
            event_rx: Mutex::new(Some(event_rx)),
            width,
            height,
            font_size: 16,
        }
    }
}

impl Backend for EguiBackend {
    type TuiBackend = RataguiBackend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>> {
        let backend = RataguiBackend::new(self.width, self.height);
        Terminal::new(backend)
    }

    fn restore_terminal(&self) -> io::Result<()> {
        Ok(())
    }

    fn enter_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        Ok(())
    }

    fn leave_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()> {
        Ok(())
    }

    fn supports_keyboard_enhancement(&self) -> bool {
        true
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn set_title<T: std::fmt::Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        title: T,
    ) -> io::Result<()> {
        Ok(())
    }

    #[cfg(feature = "clipboard")]
    fn set_clipboard<T: std::fmt::Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        content: T,
        clipboard_kind: rooibos_runtime::backend::ClipboardKind,
    ) -> io::Result<()> {
        // self.inner.set_clipboard(terminal, content, clipboard_kind)
        Ok(())
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
        while let Ok(events) = event_rx.recv().cancel_with(context.cancelled()).await {
            if let Some(events) = events {
                for event in events {
                    if let Some(event) = self.translate_event(event) {
                        let _ = term_tx
                            .send(event)
                            .inspect_err(|e| warn!("failed to send event {e:?}"));
                    }
                }
            }
        }
    }
}

impl EguiBackend {
    fn translate_event(&self, event: egui::Event) -> Option<rooibos_dom::Event> {
        match event {
            egui::Event::Copy => None,
            egui::Event::Cut => None,
            egui::Event::Paste(text) => Some(rooibos_dom::Event::Paste(text)),
            egui::Event::Text(text) => Some(rooibos_dom::Event::Paste(text)),
            egui::Event::Key {
                key,
                physical_key,
                pressed,
                repeat,
                modifiers,
            } => translate_key(physical_key.unwrap_or(key)).map(|key| {
                rooibos_dom::Event::Key(rooibos_dom::KeyEvent {
                    code: key,
                    modifiers: translate_modifiers(modifiers),
                    kind: if repeat {
                        KeyEventKind::Repeat
                    } else if pressed {
                        KeyEventKind::Press
                    } else {
                        KeyEventKind::Release
                    },
                    state: KeyEventState::empty(),
                })
            }),
            egui::Event::PointerMoved(pos) => Some(rooibos_dom::Event::Mouse(MouseEvent {
                kind: MouseEventKind::Moved,
                column: (pos.x / self.font_size as f32) as u16,
                row: (pos.y / self.font_size as f32) as u16,
                modifiers: KeyModifiers::empty(),
            })),
            egui::Event::MouseMoved(_) => None,
            egui::Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers,
            } => {
                let button = translate_mouse_button(button);
                Some(rooibos_dom::Event::Mouse(MouseEvent {
                    kind: if pressed {
                        MouseEventKind::Down(button)
                    } else {
                        MouseEventKind::Up(button)
                    },
                    column: (pos.x / self.font_size as f32) as u16,
                    row: (pos.y / self.font_size as f32) as u16,
                    modifiers: translate_modifiers(modifiers),
                }))
            }
            egui::Event::PointerGone => None,
            egui::Event::Zoom(_) => None,
            egui::Event::Ime(_) => None,
            egui::Event::Touch { .. } => None,
            egui::Event::MouseWheel { .. } => None,
            egui::Event::WindowFocused(true) => Some(rooibos_dom::Event::FocusGained),
            egui::Event::WindowFocused(false) => Some(rooibos_dom::Event::FocusLost),
            egui::Event::AccessKitActionRequest(_) => None,
            egui::Event::Screenshot { .. } => None,
        }
    }
}

fn translate_mouse_button(button: egui::PointerButton) -> rooibos_dom::MouseButton {
    match button {
        egui::PointerButton::Primary => rooibos_dom::MouseButton::Left,
        egui::PointerButton::Secondary => rooibos_dom::MouseButton::Right,
        egui::PointerButton::Middle => rooibos_dom::MouseButton::Middle,
        egui::PointerButton::Extra1 => rooibos_dom::MouseButton::Unknown,
        egui::PointerButton::Extra2 => rooibos_dom::MouseButton::Unknown,
    }
}

fn translate_modifiers(modifiers: egui::Modifiers) -> rooibos_dom::KeyModifiers {
    let mut mapped = rooibos_dom::KeyModifiers::empty();
    if modifiers.alt {
        mapped |= rooibos_dom::KeyModifiers::ALT;
    }
    if modifiers.command | modifiers.ctrl | modifiers.mac_cmd {
        mapped |= rooibos_dom::KeyModifiers::CONTROL;
    }
    if modifiers.shift {
        mapped |= rooibos_dom::KeyModifiers::SHIFT;
    }
    mapped
}

fn translate_key(key: egui::Key) -> Option<KeyCode> {
    match key {
        egui::Key::ArrowDown => Some(KeyCode::Down),
        egui::Key::ArrowLeft => Some(KeyCode::Left),
        egui::Key::ArrowRight => Some(KeyCode::Right),
        egui::Key::ArrowUp => Some(KeyCode::Up),
        egui::Key::Escape => Some(KeyCode::Esc),
        egui::Key::Tab => Some(KeyCode::Tab),
        egui::Key::Backspace => Some(KeyCode::Backspace),
        egui::Key::Enter => Some(KeyCode::Enter),
        egui::Key::Space => Some(KeyCode::Char(' ')),
        egui::Key::Insert => Some(KeyCode::Insert),
        egui::Key::Delete => Some(KeyCode::Delete),
        egui::Key::Home => Some(KeyCode::Home),
        egui::Key::End => Some(KeyCode::End),
        egui::Key::PageUp => Some(KeyCode::PageUp),
        egui::Key::PageDown => Some(KeyCode::PageDown),
        egui::Key::Copy => None,
        egui::Key::Cut => None,
        egui::Key::Paste => None,
        egui::Key::Colon => Some(KeyCode::Char(':')),
        egui::Key::Comma => Some(KeyCode::Char(',')),
        egui::Key::Backslash => Some(KeyCode::Char('\\')),
        egui::Key::Slash => Some(KeyCode::Char('/')),
        egui::Key::Pipe => Some(KeyCode::Char('|')),
        egui::Key::Questionmark => Some(KeyCode::Char('?')),
        egui::Key::OpenBracket => Some(KeyCode::Char('[')),
        egui::Key::CloseBracket => Some(KeyCode::Char(']')),
        egui::Key::Backtick => Some(KeyCode::Char('`')),
        egui::Key::Minus => Some(KeyCode::Char('-')),
        egui::Key::Period => Some(KeyCode::Char('.')),
        egui::Key::Plus => Some(KeyCode::Char('+')),
        egui::Key::Equals => Some(KeyCode::Char('=')),
        egui::Key::Semicolon => Some(KeyCode::Char(';')),
        egui::Key::Quote => Some(KeyCode::Char('\'')),
        egui::Key::Num0 => Some(KeyCode::Char('0')),
        egui::Key::Num1 => Some(KeyCode::Char('1')),
        egui::Key::Num2 => Some(KeyCode::Char('2')),
        egui::Key::Num3 => Some(KeyCode::Char('3')),
        egui::Key::Num4 => Some(KeyCode::Char('4')),
        egui::Key::Num5 => Some(KeyCode::Char('5')),
        egui::Key::Num6 => Some(KeyCode::Char('6')),
        egui::Key::Num7 => Some(KeyCode::Char('7')),
        egui::Key::Num8 => Some(KeyCode::Char('8')),
        egui::Key::Num9 => Some(KeyCode::Char('9')),
        egui::Key::A => Some(KeyCode::Char('a')),
        egui::Key::B => Some(KeyCode::Char('b')),
        egui::Key::C => Some(KeyCode::Char('c')),
        egui::Key::D => Some(KeyCode::Char('d')),
        egui::Key::E => Some(KeyCode::Char('e')),
        egui::Key::F => Some(KeyCode::Char('f')),
        egui::Key::G => Some(KeyCode::Char('g')),
        egui::Key::H => Some(KeyCode::Char('h')),
        egui::Key::I => Some(KeyCode::Char('i')),
        egui::Key::J => Some(KeyCode::Char('j')),
        egui::Key::K => Some(KeyCode::Char('k')),
        egui::Key::L => Some(KeyCode::Char('l')),
        egui::Key::M => Some(KeyCode::Char('m')),
        egui::Key::N => Some(KeyCode::Char('n')),
        egui::Key::O => Some(KeyCode::Char('o')),
        egui::Key::P => Some(KeyCode::Char('p')),
        egui::Key::Q => Some(KeyCode::Char('q')),
        egui::Key::R => Some(KeyCode::Char('r')),
        egui::Key::S => Some(KeyCode::Char('s')),
        egui::Key::T => Some(KeyCode::Char('t')),
        egui::Key::U => Some(KeyCode::Char('u')),
        egui::Key::V => Some(KeyCode::Char('v')),
        egui::Key::W => Some(KeyCode::Char('w')),
        egui::Key::X => Some(KeyCode::Char('x')),
        egui::Key::Y => Some(KeyCode::Char('y')),
        egui::Key::Z => Some(KeyCode::Char('z')),
        egui::Key::F1 => Some(KeyCode::F(1)),
        egui::Key::F2 => Some(KeyCode::F(2)),
        egui::Key::F3 => Some(KeyCode::F(3)),
        egui::Key::F4 => Some(KeyCode::F(4)),
        egui::Key::F5 => Some(KeyCode::F(5)),
        egui::Key::F6 => Some(KeyCode::F(6)),
        egui::Key::F7 => Some(KeyCode::F(7)),
        egui::Key::F8 => Some(KeyCode::F(8)),
        egui::Key::F9 => Some(KeyCode::F(9)),
        egui::Key::F10 => Some(KeyCode::F(10)),
        egui::Key::F11 => Some(KeyCode::F(11)),
        egui::Key::F12 => Some(KeyCode::F(12)),
        egui::Key::F13 => Some(KeyCode::F(13)),
        egui::Key::F14 => Some(KeyCode::F(14)),
        egui::Key::F15 => Some(KeyCode::F(15)),
        egui::Key::F16 => Some(KeyCode::F(16)),
        egui::Key::F17 => Some(KeyCode::F(17)),
        egui::Key::F18 => Some(KeyCode::F(18)),
        egui::Key::F19 => Some(KeyCode::F(19)),
        egui::Key::F20 => Some(KeyCode::F(20)),
        egui::Key::F21 => Some(KeyCode::F(21)),
        egui::Key::F22 => Some(KeyCode::F(22)),
        egui::Key::F23 => Some(KeyCode::F(23)),
        egui::Key::F24 => Some(KeyCode::F(24)),
        egui::Key::F25 => Some(KeyCode::F(25)),
        egui::Key::F26 => Some(KeyCode::F(26)),
        egui::Key::F27 => Some(KeyCode::F(27)),
        egui::Key::F28 => Some(KeyCode::F(28)),
        egui::Key::F29 => Some(KeyCode::F(29)),
        egui::Key::F30 => Some(KeyCode::F(30)),
        egui::Key::F31 => Some(KeyCode::F(31)),
        egui::Key::F32 => Some(KeyCode::F(32)),
        egui::Key::F33 => Some(KeyCode::F(33)),
        egui::Key::F34 => Some(KeyCode::F(34)),
        egui::Key::F35 => Some(KeyCode::F(35)),
    }
}
