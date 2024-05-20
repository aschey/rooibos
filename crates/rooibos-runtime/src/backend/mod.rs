use std::io;
use std::pin::Pin;
#[cfg(feature = "crossterm")]
pub mod crossterm;
#[cfg(feature = "termion")]
pub mod termion;
pub mod test;

use futures_util::Future;
use ratatui::Terminal;
use tokio::sync::{broadcast, mpsc};

pub trait Backend: Send {
    type TuiBackend: ratatui::backend::Backend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>>;

    fn restore_terminal(&self) -> io::Result<()>;

    fn supports_keyboard_enhancement(&self) -> bool;

    fn read_input(
        &self,
        quit_tx: mpsc::Sender<()>,
        term_tx: broadcast::Sender<rooibos_dom::Event>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}
