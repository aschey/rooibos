use crate::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MediaKeyCode,
    ModifierKeyCode, MouseButton, MouseEvent, MouseEventKind,
};

impl From<crossterm::event::Event> for Event {
    fn from(value: crossterm::event::Event) -> Self {
        match value {
            crossterm::event::Event::FocusGained => Event::FocusGained,
            crossterm::event::Event::FocusLost => Event::FocusLost,
            crossterm::event::Event::Key(key_event) => Event::Key(key_event.into()),
            crossterm::event::Event::Mouse(mouse_event) => Event::Mouse(mouse_event.into()),
            crossterm::event::Event::Paste(value) => Event::Paste(value),
            crossterm::event::Event::Resize(cols, rows) => Event::Resize(cols, rows),
        }
    }
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(value: crossterm::event::MouseEvent) -> Self {
        Self {
            kind: value.kind.into(),
            column: value.column,
            row: value.row,
            modifiers: value.modifiers.into(),
        }
    }
}

impl From<crossterm::event::MouseEventKind> for MouseEventKind {
    fn from(value: crossterm::event::MouseEventKind) -> Self {
        match value {
            crossterm::event::MouseEventKind::Down(button) => MouseEventKind::Down(button.into()),
            crossterm::event::MouseEventKind::Up(button) => MouseEventKind::Up(button.into()),
            crossterm::event::MouseEventKind::Drag(button) => MouseEventKind::Drag(button.into()),
            crossterm::event::MouseEventKind::Moved => MouseEventKind::Moved,
            crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
            crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
            crossterm::event::MouseEventKind::ScrollLeft => MouseEventKind::ScrollLeft,
            crossterm::event::MouseEventKind::ScrollRight => MouseEventKind::ScrollRight,
        }
    }
}

impl From<crossterm::event::MouseButton> for MouseButton {
    fn from(value: crossterm::event::MouseButton) -> Self {
        match value {
            crossterm::event::MouseButton::Left => MouseButton::Left,
            crossterm::event::MouseButton::Right => MouseButton::Right,
            crossterm::event::MouseButton::Middle => MouseButton::Middle,
        }
    }
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(value: crossterm::event::KeyEvent) -> Self {
        Self {
            code: value.code.into(),
            modifiers: value.modifiers.into(),
            kind: value.kind.into(),
            state: value.state.into(),
        }
    }
}

impl From<KeyEvent> for crossterm::event::KeyEvent {
    fn from(value: KeyEvent) -> Self {
        Self {
            code: value.code.into(),
            modifiers: value.modifiers.into(),
            kind: value.kind.into(),
            state: value.state.into(),
        }
    }
}

impl From<crossterm::event::KeyCode> for KeyCode {
    fn from(value: crossterm::event::KeyCode) -> Self {
        match value {
            crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
            crossterm::event::KeyCode::Enter => KeyCode::Enter,
            crossterm::event::KeyCode::Left => KeyCode::Left,
            crossterm::event::KeyCode::Right => KeyCode::Right,
            crossterm::event::KeyCode::Up => KeyCode::Up,
            crossterm::event::KeyCode::Down => KeyCode::Down,
            crossterm::event::KeyCode::Home => KeyCode::Home,
            crossterm::event::KeyCode::End => KeyCode::End,
            crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
            crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
            crossterm::event::KeyCode::Tab => KeyCode::Tab,
            crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
            crossterm::event::KeyCode::Delete => KeyCode::Delete,
            crossterm::event::KeyCode::Insert => KeyCode::Insert,
            crossterm::event::KeyCode::F(f) => KeyCode::F(f),
            crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
            crossterm::event::KeyCode::Null => KeyCode::Null,
            crossterm::event::KeyCode::Esc => KeyCode::Esc,
            crossterm::event::KeyCode::CapsLock => KeyCode::CapsLock,
            crossterm::event::KeyCode::ScrollLock => KeyCode::ScrollLock,
            crossterm::event::KeyCode::NumLock => KeyCode::NumLock,
            crossterm::event::KeyCode::PrintScreen => KeyCode::PrintScreen,
            crossterm::event::KeyCode::Pause => KeyCode::Pause,
            crossterm::event::KeyCode::Menu => KeyCode::Menu,
            crossterm::event::KeyCode::KeypadBegin => KeyCode::KeypadBegin,
            crossterm::event::KeyCode::Media(m) => KeyCode::Media(m.into()),
            crossterm::event::KeyCode::Modifier(m) => KeyCode::Modifier(m.into()),
        }
    }
}

impl From<KeyCode> for crossterm::event::KeyCode {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Backspace => crossterm::event::KeyCode::Backspace,
            KeyCode::Enter => crossterm::event::KeyCode::Enter,
            KeyCode::Left => crossterm::event::KeyCode::Left,
            KeyCode::Right => crossterm::event::KeyCode::Right,
            KeyCode::Up => crossterm::event::KeyCode::Up,
            KeyCode::Down => crossterm::event::KeyCode::Down,
            KeyCode::Home => crossterm::event::KeyCode::Home,
            KeyCode::End => crossterm::event::KeyCode::End,
            KeyCode::PageUp => crossterm::event::KeyCode::PageUp,
            KeyCode::PageDown => crossterm::event::KeyCode::PageDown,
            KeyCode::Tab => crossterm::event::KeyCode::Tab,
            KeyCode::BackTab => crossterm::event::KeyCode::BackTab,
            KeyCode::Delete => crossterm::event::KeyCode::Delete,
            KeyCode::Insert => crossterm::event::KeyCode::Insert,
            KeyCode::F(f) => crossterm::event::KeyCode::F(f),
            KeyCode::Char(c) => crossterm::event::KeyCode::Char(c),
            KeyCode::Null => crossterm::event::KeyCode::Null,
            KeyCode::Esc => crossterm::event::KeyCode::Esc,
            KeyCode::CapsLock => crossterm::event::KeyCode::CapsLock,
            KeyCode::ScrollLock => crossterm::event::KeyCode::ScrollLock,
            KeyCode::NumLock => crossterm::event::KeyCode::NumLock,
            KeyCode::PrintScreen => crossterm::event::KeyCode::PrintScreen,
            KeyCode::Pause => crossterm::event::KeyCode::Pause,
            KeyCode::Menu => crossterm::event::KeyCode::Menu,
            KeyCode::KeypadBegin => crossterm::event::KeyCode::KeypadBegin,
            KeyCode::Media(m) => crossterm::event::KeyCode::Media(m.into()),
            KeyCode::Modifier(m) => crossterm::event::KeyCode::Modifier(m.into()),
            KeyCode::Unknown => unreachable!(),
        }
    }
}

impl From<crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(value: crossterm::event::KeyModifiers) -> Self {
        Self::from_bits_retain(value.bits())
    }
}

impl From<KeyModifiers> for crossterm::event::KeyModifiers {
    fn from(value: KeyModifiers) -> Self {
        Self::from_bits_retain(value.bits())
    }
}

impl From<crossterm::event::KeyEventKind> for KeyEventKind {
    fn from(value: crossterm::event::KeyEventKind) -> Self {
        match value {
            crossterm::event::KeyEventKind::Press => KeyEventKind::Press,
            crossterm::event::KeyEventKind::Repeat => KeyEventKind::Repeat,
            crossterm::event::KeyEventKind::Release => KeyEventKind::Release,
        }
    }
}

impl From<KeyEventKind> for crossterm::event::KeyEventKind {
    fn from(value: KeyEventKind) -> Self {
        match value {
            KeyEventKind::Press => crossterm::event::KeyEventKind::Press,
            KeyEventKind::Repeat => crossterm::event::KeyEventKind::Repeat,
            KeyEventKind::Release => crossterm::event::KeyEventKind::Release,
        }
    }
}

impl From<crossterm::event::MediaKeyCode> for MediaKeyCode {
    fn from(value: crossterm::event::MediaKeyCode) -> Self {
        match value {
            crossterm::event::MediaKeyCode::Play => MediaKeyCode::Play,
            crossterm::event::MediaKeyCode::Pause => MediaKeyCode::Pause,
            crossterm::event::MediaKeyCode::PlayPause => MediaKeyCode::PlayPause,
            crossterm::event::MediaKeyCode::Reverse => MediaKeyCode::Reverse,
            crossterm::event::MediaKeyCode::Stop => MediaKeyCode::Stop,
            crossterm::event::MediaKeyCode::FastForward => MediaKeyCode::FastForward,
            crossterm::event::MediaKeyCode::Rewind => MediaKeyCode::Rewind,
            crossterm::event::MediaKeyCode::TrackNext => MediaKeyCode::TrackNext,
            crossterm::event::MediaKeyCode::TrackPrevious => MediaKeyCode::TrackPrevious,
            crossterm::event::MediaKeyCode::Record => MediaKeyCode::Record,
            crossterm::event::MediaKeyCode::LowerVolume => MediaKeyCode::LowerVolume,
            crossterm::event::MediaKeyCode::RaiseVolume => MediaKeyCode::RaiseVolume,
            crossterm::event::MediaKeyCode::MuteVolume => MediaKeyCode::MuteVolume,
        }
    }
}

impl From<MediaKeyCode> for crossterm::event::MediaKeyCode {
    fn from(value: MediaKeyCode) -> Self {
        match value {
            MediaKeyCode::Play => crossterm::event::MediaKeyCode::Play,
            MediaKeyCode::Pause => crossterm::event::MediaKeyCode::Pause,
            MediaKeyCode::PlayPause => crossterm::event::MediaKeyCode::PlayPause,
            MediaKeyCode::Reverse => crossterm::event::MediaKeyCode::Reverse,
            MediaKeyCode::Stop => crossterm::event::MediaKeyCode::Stop,
            MediaKeyCode::FastForward => crossterm::event::MediaKeyCode::FastForward,
            MediaKeyCode::Rewind => crossterm::event::MediaKeyCode::Rewind,
            MediaKeyCode::TrackNext => crossterm::event::MediaKeyCode::TrackNext,
            MediaKeyCode::TrackPrevious => crossterm::event::MediaKeyCode::TrackPrevious,
            MediaKeyCode::Record => crossterm::event::MediaKeyCode::Record,
            MediaKeyCode::LowerVolume => crossterm::event::MediaKeyCode::LowerVolume,
            MediaKeyCode::RaiseVolume => crossterm::event::MediaKeyCode::RaiseVolume,
            MediaKeyCode::MuteVolume => crossterm::event::MediaKeyCode::MuteVolume,
        }
    }
}

impl From<crossterm::event::ModifierKeyCode> for ModifierKeyCode {
    fn from(value: crossterm::event::ModifierKeyCode) -> Self {
        match value {
            crossterm::event::ModifierKeyCode::LeftShift => ModifierKeyCode::LeftShift,
            crossterm::event::ModifierKeyCode::LeftControl => ModifierKeyCode::LeftControl,
            crossterm::event::ModifierKeyCode::LeftAlt => ModifierKeyCode::LeftAlt,
            crossterm::event::ModifierKeyCode::LeftSuper => ModifierKeyCode::LeftSuper,
            crossterm::event::ModifierKeyCode::LeftHyper => ModifierKeyCode::LeftHyper,
            crossterm::event::ModifierKeyCode::LeftMeta => ModifierKeyCode::LeftMeta,
            crossterm::event::ModifierKeyCode::RightShift => ModifierKeyCode::RightShift,
            crossterm::event::ModifierKeyCode::RightControl => ModifierKeyCode::RightControl,
            crossterm::event::ModifierKeyCode::RightAlt => ModifierKeyCode::RightAlt,
            crossterm::event::ModifierKeyCode::RightSuper => ModifierKeyCode::RightSuper,
            crossterm::event::ModifierKeyCode::RightHyper => ModifierKeyCode::RightHyper,
            crossterm::event::ModifierKeyCode::RightMeta => ModifierKeyCode::RightMeta,
            crossterm::event::ModifierKeyCode::IsoLevel3Shift => ModifierKeyCode::IsoLevel3Shift,
            crossterm::event::ModifierKeyCode::IsoLevel5Shift => ModifierKeyCode::IsoLevel5Shift,
        }
    }
}

impl From<ModifierKeyCode> for crossterm::event::ModifierKeyCode {
    fn from(value: ModifierKeyCode) -> Self {
        match value {
            ModifierKeyCode::LeftShift => crossterm::event::ModifierKeyCode::LeftShift,
            ModifierKeyCode::LeftControl => crossterm::event::ModifierKeyCode::LeftControl,
            ModifierKeyCode::LeftAlt => crossterm::event::ModifierKeyCode::LeftAlt,
            ModifierKeyCode::LeftSuper => crossterm::event::ModifierKeyCode::LeftSuper,
            ModifierKeyCode::LeftHyper => crossterm::event::ModifierKeyCode::LeftHyper,
            ModifierKeyCode::LeftMeta => crossterm::event::ModifierKeyCode::LeftMeta,
            ModifierKeyCode::RightShift => crossterm::event::ModifierKeyCode::RightShift,
            ModifierKeyCode::RightControl => crossterm::event::ModifierKeyCode::RightControl,
            ModifierKeyCode::RightAlt => crossterm::event::ModifierKeyCode::RightAlt,
            ModifierKeyCode::RightSuper => crossterm::event::ModifierKeyCode::RightSuper,
            ModifierKeyCode::RightHyper => crossterm::event::ModifierKeyCode::RightHyper,
            ModifierKeyCode::RightMeta => crossterm::event::ModifierKeyCode::RightMeta,
            ModifierKeyCode::IsoLevel3Shift => crossterm::event::ModifierKeyCode::IsoLevel3Shift,
            ModifierKeyCode::IsoLevel5Shift => crossterm::event::ModifierKeyCode::IsoLevel5Shift,
        }
    }
}

impl From<crossterm::event::KeyEventState> for KeyEventState {
    fn from(value: crossterm::event::KeyEventState) -> Self {
        Self::from_bits_retain(value.bits())
    }
}

impl From<KeyEventState> for crossterm::event::KeyEventState {
    fn from(value: KeyEventState) -> Self {
        Self::from_bits_retain(value.bits())
    }
}
