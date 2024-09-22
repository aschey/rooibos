#[cfg(all(feature = "crossterm", not(target_arch = "wasm32")))]
pub mod crossterm;
#[cfg(all(feature = "termion", not(target_arch = "wasm32")))]
pub mod termion;
#[cfg(all(feature = "termwiz", not(target_arch = "wasm32")))]
pub mod termwiz;
pub mod test;

use std::fmt::Display;
use std::io;

use futures_util::Stream;
use ratatui::Terminal;
use tokio::sync::broadcast;

// From https://github.com/crossterm-rs/crossterm/pull/697
/// Which selection to set. Only affects X11. See
/// [X Window selection](https://en.wikipedia.org/wiki/X_Window_selection) for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardKind {
    /// Set the clipboard selection. This is the only clipboard in most windowing systems.
    /// In X11, it's the selection set by an explicit copy command
    Clipboard,
    /// Set the primary selection.
    /// In windowing systems other than X11, terminals often perform the same behavior
    /// as with Clipboard for Primary.
    /// In X11, this sets the selection used when text is highlighted.
    Primary,
    // XTerm also supports "secondary", "select", and "cut-buffers" 0-7 as kinds.
    // Since those aren't supported elsewhere, not exposing those from here
}

#[cfg(not(target_arch = "wasm32"))]
pub trait AsyncInputStream: Stream<Item = rooibos_dom::Event> + Send + 'static {}

#[cfg(target_arch = "wasm32")]
pub trait AsyncInputStream: Stream<Item = rooibos_dom::Event> + 'static {}

#[cfg(not(target_arch = "wasm32"))]
impl<T> AsyncInputStream for T where T: Stream<Item = rooibos_dom::Event> + Send + 'static {}

#[cfg(target_arch = "wasm32")]
impl<T> AsyncInputStream for T where T: Stream<Item = rooibos_dom::Event> + 'static {}

pub trait Backend: Send + Sync {
    type TuiBackend: ratatui::backend::Backend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>>;

    fn restore_terminal(&self) -> io::Result<()>;

    fn supports_keyboard_enhancement(&self) -> bool;

    fn enter_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()>;

    fn leave_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()>;

    fn set_title<T: Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        title: T,
    ) -> io::Result<()>;

    fn set_clipboard<T: Display>(
        &self,
        terminal: &mut Terminal<Self::TuiBackend>,
        content: T,
        clipboard_kind: ClipboardKind,
    ) -> io::Result<()>;

    fn supports_async_input(&self) -> bool {
        true
    }

    fn poll_input(
        &self,
        _terminal: &mut Terminal<Self::TuiBackend>,
        _term_tx: &broadcast::Sender<rooibos_dom::Event>,
    ) -> io::Result<()> {
        Ok(())
    }

    fn async_input_stream(&self) -> impl AsyncInputStream;

    fn write_all(&self, buf: &[u8]) -> io::Result<()>;
}
