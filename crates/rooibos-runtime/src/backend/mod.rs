#[cfg(all(feature = "crossterm", not(target_arch = "wasm32")))]
pub mod crossterm;
#[cfg(all(feature = "termion", not(target_arch = "wasm32")))]
pub mod termion;
#[cfg(all(feature = "termwiz", not(target_arch = "wasm32")))]
pub mod termwiz;
pub mod test;

use std::{future, io};

use futures_util::Future;
use ratatui::Terminal;
use tokio::sync::broadcast;

pub trait Backend: Send + Sync {
    type TuiBackend: ratatui::backend::Backend;

    fn setup_terminal(&self) -> io::Result<Terminal<Self::TuiBackend>>;

    fn restore_terminal(&self) -> io::Result<()>;

    fn supports_keyboard_enhancement(&self) -> bool;

    fn enter_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()>;

    fn leave_alt_screen(&self, terminal: &mut Terminal<Self::TuiBackend>) -> io::Result<()>;

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

    #[cfg(not(target_arch = "wasm32"))]
    fn read_input(
        &self,
        _term_tx: broadcast::Sender<rooibos_dom::Event>,
    ) -> impl Future<Output = ()> + Send {
        future::ready(())
    }

    #[cfg(target_arch = "wasm32")]
    fn read_input(
        &self,
        _term_tx: broadcast::Sender<rooibos_dom::Event>,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }

    fn write_all(&self, buf: &[u8]) -> io::Result<()>;
}
