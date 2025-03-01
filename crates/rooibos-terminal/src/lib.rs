#[cfg(all(feature = "crossterm", not(target_arch = "wasm32")))]
pub mod crossterm;
mod stream;
#[cfg(all(feature = "termion", not(target_arch = "wasm32")))]
pub mod termion;
#[cfg(all(feature = "termwiz", not(target_arch = "wasm32")))]
pub mod termwiz;
pub mod test;

use std::fmt::Display;
use std::{env, io};

use futures_util::Stream;
pub use stream::*;
use tokio::sync::broadcast;

#[cfg(all(feature = "crossterm", not(target_arch = "wasm32")))]
pub type DefaultBackend<T> = crossterm::CrosstermBackend<T>;

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

    fn window_size(&self) -> io::Result<ratatui::backend::WindowSize>;

    fn set_title<T: Display>(&self, backend: &mut Self::TuiBackend, title: T) -> io::Result<()>;

    fn set_clipboard<T: Display>(
        &self,
        backend: &mut Self::TuiBackend,
        content: T,
        clipboard_kind: ClipboardKind,
    ) -> io::Result<()>;

    fn supports_async_input(&self) -> bool {
        true
    }

    #[allow(unused)]
    fn poll_input(
        &self,
        terminal: &mut Self::TuiBackend,
        term_tx: &broadcast::Sender<rooibos_dom::Event>,
    ) -> io::Result<()> {
        Ok(())
    }

    fn async_input_stream(&self) -> impl AsyncInputStream;

    fn write_all(&self, buf: &[u8]) -> io::Result<()>;
}

fn parse_env_var(var_name: &str) -> bool {
    let Ok(env_var) = env::var(var_name) else {
        return false;
    };
    let env_var = env_var.to_ascii_lowercase();
    env_var == "1" || env_var == "true"
}

fn color_override() -> Option<bool> {
    if parse_env_var("NO_COLOR") {
        Some(false)
    } else if parse_env_var("CLICOLOR_FORCE") || parse_env_var("FORCE_COLOR") {
        Some(true)
    } else {
        None
    }
}

#[cfg(feature = "crossterm")]
fn adjust_color_output<T>(writer: &T)
where
    T: std::io::IsTerminal,
{
    if let Some(set_override) = color_override() {
        ::crossterm::style::force_color_output(set_override);
    } else if !writer.is_terminal() {
        ::crossterm::style::force_color_output(false);
    }
}
