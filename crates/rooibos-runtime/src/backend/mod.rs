#[cfg(feature = "crossterm")]
pub mod crossterm;
#[cfg(feature = "termion")]
pub mod termion;
pub mod test;

use std::io;

use futures_util::Future;
use ratatui::Terminal;
use tokio::sync::{broadcast, mpsc};

use crate::SignalMode;

pub trait Backend: Send + Sync {
    type TuiBackend: ratatui::backend::Backend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>>;

    fn restore_terminal(&self) -> io::Result<()>;

    fn supports_keyboard_enhancement(&self) -> bool;

    fn read_input(
        &self,
        signal_tx: mpsc::Sender<SignalMode>,
        term_tx: broadcast::Sender<rooibos_dom::Event>,
    ) -> impl Future<Output = ()> + Send;
}
