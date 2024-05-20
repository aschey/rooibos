#[cfg(feature = "crossterm")]
mod crossterm;
mod event_handler;
mod key;
mod mouse;
#[cfg(feature = "termion")]
mod termion;

pub(crate) use event_handler::*;
pub use key::*;
pub use mouse::*;
use ratatui::layout::Rect;

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Hash)]
pub enum Event {
    /// The terminal gained focus
    FocusGained,
    /// The terminal lost focus
    FocusLost,
    /// A single key event with additional pressed modifiers.
    Key(KeyEvent),
    /// A single mouse event with additional pressed modifiers.
    Mouse(MouseEvent),
    /// A string that was pasted into the terminal. Only emitted if bracketed paste has been
    /// enabled.
    Paste(String),
    /// An resize event with new dimensions after resize (columns, rows).
    /// **Note** that resize events can occur in batches.
    Resize(u16, u16),
    Unknown,
}

pub struct EventData {
    pub rect: Rect,
}
