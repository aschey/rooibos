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
    (BidiChannel { tx: tx1, rx: rx2 }, BidiChannel {
        tx: tx2,
        rx: rx1,
    })
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

enum TermResponse {
    WindowSize(io::Result<WindowSize>),
    Size(io::Result<Size>),
    Empty,
}

pub struct NonblockingTerminal<B>
where
    B: Backend,
{
    terminal: Arc<RwLock<ratatui::Terminal<B>>>,
    requester: BidiChannel<TermRequest, TermResponse>,
    handle: JoinHandle<()>,
}

impl<B> NonblockingTerminal<B>
where
    B: Backend + Send + Sync + 'static,
{
    pub fn new(terminal: ratatui::Terminal<B>) -> Self {
        let terminal = Arc::new(RwLock::new(terminal));
        let (requester, mut handle) = channel::<TermRequest, TermResponse>(1);

        let handle = tokio::task::spawn_blocking({
            let terminal = terminal.clone();
            move || {
                while let Some(req) = handle.rx.blocking_recv() {
                    match req {
                        TermRequest::WindowSize => {
                            handle
                                .tx
                                .blocking_send(TermResponse::WindowSize(
                                    terminal.write().backend_mut().window_size(),
                                ))
                                .unwrap();
                        }
                        TermRequest::AutoResize => {
                            terminal.write().autoresize().unwrap();
                            handle.tx.blocking_send(TermResponse::Empty).unwrap();
                        }
                        TermRequest::Draw => {
                            let mut terminal = terminal.write();
                            terminal.flush().unwrap();
                            // terminal.hide_cursor()?;
                            // match cursor_position {
                            //     None => terminal.hide_cursor()?,
                            //     Some(position) => {
                            //         terminal.show_cursor()?;
                            //         terminal.set_cursor_position(position)?;
                            //     }
                            // }

                            terminal.swap_buffers();

                            // Flush
                            ratatui::backend::Backend::flush(terminal.backend_mut()).unwrap();
                            handle.tx.blocking_send(TermResponse::Empty).unwrap();
                        }
                        TermRequest::Size => {
                            handle
                                .tx
                                .blocking_send(TermResponse::Size(terminal.read().size()))
                                .unwrap();
                        }
                        TermRequest::SetCursorPosition(position) => {
                            terminal.write().set_cursor_position(position).unwrap();
                            handle.tx.blocking_send(TermResponse::Empty).unwrap();
                        }
                        TermRequest::Clear => {
                            terminal.write().clear().unwrap();
                            handle.tx.blocking_send(TermResponse::Empty).unwrap();
                        }
                        TermRequest::InsertText { height, text } => {
                            terminal
                                .write()
                                .insert_before(height, |buf| {
                                    Paragraph::new(text).render(buf.area, buf);
                                })
                                .unwrap();
                            handle.tx.blocking_send(TermResponse::Empty).unwrap();
                        }
                    }
                }
            }
        });

        Self {
            terminal,
            requester,
            handle,
        }
    }

    pub async fn window_size(&mut self) -> io::Result<WindowSize> {
        self.requester
            .tx
            .send(TermRequest::WindowSize)
            .await
            .unwrap();
        let TermResponse::WindowSize(window_size) = self.requester.rx.recv().await.unwrap() else {
            unreachable!()
        };
        window_size
    }

    pub async fn auto_resize(&mut self) {
        self.requester
            .tx
            .send(TermRequest::AutoResize)
            .await
            .unwrap();
        self.requester.rx.recv().await.unwrap();
    }

    pub fn with_frame_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Frame) -> R,
    {
        f(&mut self.terminal.write().get_frame())
    }

    pub async fn draw(&mut self) {
        self.requester.tx.send(TermRequest::Draw).await.unwrap();
        self.requester.rx.recv().await.unwrap();
    }

    pub fn area(&self) -> Rect {
        self.terminal.write().current_buffer_mut().area
    }

    pub async fn size(&mut self) -> Result<Size, io::Error> {
        self.requester.tx.send(TermRequest::Size).await.unwrap();
        let TermResponse::Size(size) = self.requester.rx.recv().await.unwrap() else {
            unreachable!()
        };
        size
    }

    pub async fn set_cursor_position(&mut self, position: Position) {
        self.requester
            .tx
            .send(TermRequest::SetCursorPosition(position))
            .await
            .unwrap();
        self.requester.rx.recv().await.unwrap();
    }

    pub async fn clear(&mut self) {
        self.requester.tx.send(TermRequest::Clear).await.unwrap();
        self.requester.rx.recv().await.unwrap();
    }

    pub async fn insert_before(&mut self, height: u16, text: Text<'static>) {
        self.requester
            .tx
            .send(TermRequest::InsertText { height, text })
            .await
            .unwrap();
        self.requester.rx.recv().await.unwrap();
    }

    pub fn with_terminal_mut<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut Terminal<B>) -> R,
    {
        f(&mut self.terminal.write())
    }

    pub async fn join(self) {
        drop(self.requester);
        self.handle.await.unwrap();
    }
}
