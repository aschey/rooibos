use std::io;
use std::sync::Arc;

use ratatui::backend::WindowSize;
use ratatui::layout::{Position, Rect, Size};
use ratatui::prelude::Backend;
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget as _};
use ratatui::{Frame, Terminal};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use wasm_compat::sync::RwLock;

pub(crate) struct BidiChannel<S, R> {
    pub(crate) tx: mpsc::Sender<S>,
    pub(crate) rx: mpsc::Receiver<R>,
}

pub(crate) fn channel<S, R>(buffer_size: usize) -> (BidiChannel<S, R>, BidiChannel<R, S>) {
    let (tx1, rx1) = mpsc::channel(buffer_size);
    let (tx2, rx2) = mpsc::channel(buffer_size);
    (
        BidiChannel { tx: tx1, rx: rx2 },
        BidiChannel { tx: tx2, rx: rx1 },
    )
}

enum TermRequest {
    WindowSize,
    AutoResize,
    Draw,
    Size,
    SetCursorPosition(Position),
    Clear,
    InsertText { height: u16, text: Text<'static> },
}

pub struct NonblockingTerminal<B>
where
    B: Backend,
{
    terminal: ratatui::Terminal<B>,
}

impl<B> NonblockingTerminal<B>
where
    B: Backend + 'static,
{
    pub fn new(terminal: ratatui::Terminal<B>) -> Self {
        Self { terminal }
    }

    pub async fn window_size(&mut self) -> Result<WindowSize, B::Error> {
        self.terminal.backend_mut().window_size()
    }

    pub async fn auto_resize(&mut self) {
        self.terminal.autoresize().unwrap();
    }

    pub fn with_frame_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Frame) -> R,
    {
        f(&mut self.terminal.get_frame())
    }

    pub async fn draw(&mut self) {
        self.terminal.flush().unwrap();

        self.terminal.swap_buffers();

        // Flush
        ratatui::backend::Backend::flush(self.terminal.backend_mut()).unwrap();
    }

    pub fn area(&mut self) -> Rect {
        self.terminal.current_buffer_mut().area
    }

    pub async fn size(&mut self) -> Result<Size, B::Error> {
        self.terminal.size()
    }

    pub async fn set_cursor_position(&mut self, position: Position) {
        self.terminal.set_cursor_position(position).unwrap()
    }

    pub async fn clear(&mut self) {
        self.terminal.clear().unwrap();
    }

    pub async fn insert_before(&mut self, height: u16, text: Text<'static>) {
        self.terminal
            .insert_before(height, |buf| {
                Paragraph::new(text).render(buf.area, buf);
            })
            .unwrap();
    }

    pub async fn with_terminal_mut<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut Terminal<B>) -> R + 'static,
    {
        f(&mut self.terminal)
    }

    pub async fn with_terminal<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&Terminal<B>) -> R + 'static,
    {
        f(&self.terminal)
    }

    pub async fn join(self) {}
}
