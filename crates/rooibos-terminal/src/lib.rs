mod stream;
#[cfg(all(feature = "termina", not(target_arch = "wasm32")))]
pub mod termina;

use std::fmt::Display;
use std::io;

use futures_util::Stream;
pub use stream::*;
use tokio_util::sync::CancellationToken;
use tui_theme::ColorPalette;

#[cfg(all(feature = "termina", not(target_arch = "wasm32")))]
pub type DefaultBackend<T> = termina::TerminaBackend<T>;

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

    fn create_tui_backend(&self) -> io::Result<Self::TuiBackend>;

    fn setup_terminal(&self, backend: &mut Self::TuiBackend) -> io::Result<()>;

    fn restore_terminal(&self) -> io::Result<()>;

    fn supports_keyboard_enhancement(&self) -> bool;

    fn enter_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()>;

    fn leave_alt_screen(&self, backend: &mut Self::TuiBackend) -> io::Result<()>;

    fn window_size(
        &self,
        backend: &mut Self::TuiBackend,
    ) -> io::Result<ratatui::backend::WindowSize>;

    fn set_title<T: Display>(&self, backend: &mut Self::TuiBackend, title: T) -> io::Result<()>;

    fn set_clipboard<T: Display>(
        &self,
        backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: ClipboardKind,
    ) -> io::Result<()>;

    fn color_palette(&self) -> ColorPalette;

    fn profile(&self) -> tui_theme::TermProfile;

    fn async_input_stream(&self, cancellation_token: CancellationToken) -> impl AsyncInputStream;

    fn write_all(&self, buf: &[u8]) -> io::Result<()>;
}
