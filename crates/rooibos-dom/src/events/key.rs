use std::hash::{Hash, Hasher};

use bitflags::bitflags;

#[derive(Debug, PartialOrd, Clone, Copy)]
pub struct KeyEvent {
    /// The key itself.
    pub code: KeyCode,
    /// Additional key modifiers.
    pub modifiers: KeyModifiers,
    /// Kind of event.
    ///
    /// Only set if:
    /// - Unix: [`KeyboardEnhancementFlags::REPORT_EVENT_TYPES`] has been enabled with
    ///   [`PushKeyboardEnhancementFlags`].
    /// - Windows: always
    pub kind: KeyEventKind,
    /// Keyboard state.
    ///
    /// Only set if [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    pub state: KeyEventState,
}

impl KeyEvent {
    // modifies the KeyEvent,
    // so that KeyModifiers::SHIFT is present iff
    // an uppercase char is present.
    fn normalize_case(mut self) -> KeyEvent {
        let c = match self.code {
            KeyCode::Char(c) => c,
            _ => return self,
        };

        if c.is_ascii_uppercase() {
            self.modifiers.insert(KeyModifiers::SHIFT);
        } else if self.modifiers.contains(KeyModifiers::SHIFT) {
            self.code = KeyCode::Char(c.to_ascii_uppercase())
        }
        self
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

impl PartialEq for KeyEvent {
    fn eq(&self, other: &KeyEvent) -> bool {
        let KeyEvent {
            code: lhs_code,
            modifiers: lhs_modifiers,
            kind: lhs_kind,
            state: lhs_state,
        } = self.normalize_case();
        let KeyEvent {
            code: rhs_code,
            modifiers: rhs_modifiers,
            kind: rhs_kind,
            state: rhs_state,
        } = other.normalize_case();
        (lhs_code == rhs_code)
            && (lhs_modifiers == rhs_modifiers)
            && (lhs_kind == rhs_kind)
            && (lhs_state == rhs_state)
    }
}

impl Eq for KeyEvent {}

impl Hash for KeyEvent {
    fn hash<H: Hasher>(&self, hash_state: &mut H) {
        let KeyEvent {
            code,
            modifiers,
            kind,
            state,
        } = self.normalize_case();
        code.hash(hash_state);
        modifiers.hash(hash_state);
        kind.hash(hash_state);
        state.hash(hash_state);
    }
}

/// Represents a key.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyCode {
    /// Backspace key.
    Backspace,
    /// Enter key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page down key.
    PageDown,
    /// Tab key.
    Tab,
    /// Shift + Tab key.
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// F key.
    ///
    /// `KeyCode::F(1)` represents F1 key, etc.
    F(u8),
    /// A character.
    ///
    /// `KeyCode::Char('c')` represents `c` character, etc.
    Char(char),
    /// Null.
    Null,
    /// Escape key.
    Esc,
    /// Caps Lock key.
    ///
    /// **Note:** this key can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    CapsLock,
    /// Scroll Lock key.
    ///
    /// **Note:** this key can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    ScrollLock,
    /// Num Lock key.
    ///
    /// **Note:** this key can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    NumLock,
    /// Print Screen key.
    ///
    /// **Note:** this key can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    PrintScreen,
    /// Pause key.
    ///
    /// **Note:** this key can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    Pause,
    /// Menu key.
    ///
    /// **Note:** this key can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    Menu,
    /// The "Begin" key (often mapped to the 5 key when Num Lock is turned on).
    ///
    /// **Note:** this key can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    KeypadBegin,
    /// A media key.
    ///
    /// **Note:** these keys can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    Media(MediaKeyCode),
    /// A modifier key.
    ///
    /// **Note:** these keys can only be read if **both**
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] and
    /// [`KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES`] have been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    Modifier(ModifierKeyCode),
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
        }
    }
}

bitflags! {
    /// Represents key modifiers (shift, control, alt, etc.).
    ///
    /// **Note:** `SUPER`, `HYPER`, and `META` can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
        const NONE = 0b0000_0000;
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

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyEventKind {
    Press,
    Repeat,
    Release,
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

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MediaKeyCode {
    /// Play media key.
    Play,
    /// Pause media key.
    Pause,
    /// Play/Pause media key.
    PlayPause,
    /// Reverse media key.
    Reverse,
    /// Stop media key.
    Stop,
    /// Fast-forward media key.
    FastForward,
    /// Rewind media key.
    Rewind,
    /// Next-track media key.
    TrackNext,
    /// Previous-track media key.
    TrackPrevious,
    /// Record media key.
    Record,
    /// Lower-volume media key.
    LowerVolume,
    /// Raise-volume media key.
    RaiseVolume,
    /// Mute media key.
    MuteVolume,
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

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ModifierKeyCode {
    /// Left Shift key.
    LeftShift,
    /// Left Control key.
    LeftControl,
    /// Left Alt key.
    LeftAlt,
    /// Left Super key.
    LeftSuper,
    /// Left Hyper key.
    LeftHyper,
    /// Left Meta key.
    LeftMeta,
    /// Right Shift key.
    RightShift,
    /// Right Control key.
    RightControl,
    /// Right Alt key.
    RightAlt,
    /// Right Super key.
    RightSuper,
    /// Right Hyper key.
    RightHyper,
    /// Right Meta key.
    RightMeta,
    /// Iso Level3 Shift key.
    IsoLevel3Shift,
    /// Iso Level5 Shift key.
    IsoLevel5Shift,
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

bitflags! {
    /// Represents extra state about the key event.
    ///
    /// **Note:** This state can only be read if
    /// [`KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES`] has been enabled with
    /// [`PushKeyboardEnhancementFlags`].
    #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct KeyEventState: u8 {
        /// The key event origins from the keypad.
        const KEYPAD = 0b0000_0001;
        /// Caps Lock was enabled for this key event.
        ///
        /// **Note:** this is set for the initial press of Caps Lock itself.
        const CAPS_LOCK = 0b0000_1000;
        /// Num Lock was enabled for this key event.
        ///
        /// **Note:** this is set for the initial press of Num Lock itself.
        const NUM_LOCK = 0b0000_1000;
        const NONE = 0b0000_0000;
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
