use std::fmt::Write;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Terminal;
use rooibos_dom::{
    focus_next, render_dom, DomNodeRepr, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton,
    MouseEvent, MouseEventKind, NodeTypeRepr, Render,
};
use rooibos_runtime::backend::test::TestBackend;
use rooibos_runtime::wasm_compat::{Lazy, RwLock};
use rooibos_runtime::{once, Runtime, RuntimeSettings, TickResult};
use tokio::sync::broadcast;
use unicode_width::UnicodeWidthStr;

once! {
    static DEFAULT_TIMEOUT: Lazy<RwLock<Duration>> =
        Lazy::new(move || RwLock::new(Duration::from_secs(1)));
}

pub trait TerminalView {
    fn terminal_view(&self) -> String;
    fn slice(&self, rect: Rect) -> Buffer;
}

impl TerminalView for Buffer {
    fn terminal_view(&self) -> String {
        let Rect { width, height, .. } = self.area();

        let mut string_buf = String::with_capacity((width * height) as usize);
        for row in 0..*height {
            for col in 0..*width {
                let cell = self.get(col, row);
                write!(&mut string_buf, "{}", cell.symbol()).unwrap();
            }
            writeln!(&mut string_buf).unwrap();
        }
        string_buf
    }

    fn slice(&self, rect: Rect) -> Buffer {
        let mut buf = Buffer::empty(Rect {
            x: 0,
            y: 0,
            width: rect.width,
            height: rect.height,
        });
        for row in rect.y..rect.y + rect.height {
            for col in rect.x..rect.x + rect.width {
                let cur = self.get(col, row);
                let new = buf.get_mut(col - rect.x, row - rect.y);

                new.set_skip(cur.skip);
                new.set_style(cur.style());
                new.set_symbol(cur.symbol());
            }
        }
        buf
    }
}

pub struct TestHarness {
    runtime: Runtime<TestBackend>,
    terminal: Terminal<ratatui::backend::TestBackend>,
    event_tx: broadcast::Sender<Event>,
}

impl TestHarness {
    pub fn set_default_timeout(timeout: Duration) {
        DEFAULT_TIMEOUT.with(|t| t.with_mut(|d| *d = timeout));
    }

    pub fn new<F, M>(runtime_settings: RuntimeSettings, width: u16, height: u16, f: F) -> Self
    where
        F: FnOnce() -> M + 'static,
        M: Render,
    {
        let backend = TestBackend::new(width, height);
        let event_tx = backend.event_tx();
        let mut runtime = Runtime::initialize(runtime_settings, backend, f);
        let mut terminal = runtime.setup_terminal().unwrap();
        terminal.draw(|f| render_dom(f.buffer_mut())).unwrap();
        focus_next();

        Self {
            runtime,
            terminal,
            event_tx,
        }
    }

    pub fn terminal(&self) -> &Terminal<ratatui::backend::TestBackend> {
        &self.terminal
    }

    pub fn buffer(&self) -> &Buffer {
        self.terminal.backend().buffer()
    }

    pub async fn wait_for(&mut self, f: impl FnMut(&Buffer, &Self) -> bool) -> Result<(), Buffer> {
        DEFAULT_TIMEOUT
            .with(|t| t.with(|t| self.wait_for_timeout(f, *t)))
            .await
    }

    pub fn find_by_text(&self, node: &DomNodeRepr, text: impl AsRef<str>) -> Option<DomNodeRepr> {
        let text = text.as_ref();
        let buf = self.buffer();
        node.find(|node| {
            node.node_type() == NodeTypeRepr::Widget
                && buf.slice(node.rect()).terminal_view().contains(text)
        })
    }

    pub fn find_by_block_text(
        &self,
        node: &DomNodeRepr,
        text: impl AsRef<str>,
    ) -> Option<DomNodeRepr> {
        let text = text.as_ref();
        let buf = self.buffer();
        node.find(|node| {
            if let NodeTypeRepr::Layout(props) = node.node_type() {
                if props.block.is_some() {
                    let rect = node.rect();

                    // Check top of block
                    if buf
                        .slice(Rect {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: 1,
                        })
                        .terminal_view()
                        .contains(text)
                    {
                        return true;
                    }

                    // Check bottom of block
                    if buf
                        .slice(Rect {
                            x: rect.x,
                            y: rect.y + rect.height - 1,
                            width: rect.width,
                            height: 1,
                        })
                        .terminal_view()
                        .contains(text)
                    {
                        return true;
                    }
                }
            }
            false
        })
    }

    pub fn get_by_text(&self, node: &DomNodeRepr, text: impl AsRef<str>) -> DomNodeRepr {
        if let Some(node) = self.find_by_text(node, text) {
            node
        } else {
            panic!("{}", self.buffer().terminal_view());
        }
    }

    pub fn get_by_block_text(&self, node: &DomNodeRepr, text: impl AsRef<str>) -> DomNodeRepr {
        if let Some(node) = self.find_by_block_text(node, text) {
            node
        } else {
            panic!("{}", self.buffer().terminal_view());
        }
    }

    pub fn find_all_by_text(&self, node: &DomNodeRepr, text: impl AsRef<str>) -> Vec<DomNodeRepr> {
        let text = text.as_ref();
        node.find_all(|node| {
            node.node_type() == NodeTypeRepr::Widget
                && self
                    .buffer()
                    .slice(node.rect())
                    .terminal_view()
                    .contains(text)
        })
    }

    pub fn send_event(&self, event: Event) {
        self.event_tx.send(event).unwrap();
    }

    pub fn send_text(&self, text: impl Into<String>) {
        let text = text.into();
        for char in text.chars() {
            self.send_event(Event::Key(KeyCode::Char(char).into()));
        }
    }

    pub fn send_key(&self, key_code: KeyCode) {
        self.send_event(Event::Key(key_code.into()));
    }

    pub fn send_mouse_move(&self, x: u16, y: u16) {
        self.send_event(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: x,
            row: y,
            modifiers: KeyModifiers::empty(),
        }));
    }

    pub fn click_pos(&self, rect: Rect) {
        self.send_event(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            row: rect.y,
            column: rect.x,
            modifiers: KeyModifiers::empty(),
        }));
    }

    pub async fn wait_for_timeout(
        &mut self,
        mut f: impl FnMut(&Buffer, &Self) -> bool,
        timeout: Duration,
    ) -> Result<(), Buffer> {
        let start = Instant::now();
        loop {
            tokio::select! {
                tick_result = self.runtime.tick() => {
                    match tick_result {
                        TickResult::Redraw => {
                            self.terminal.draw(|f| render_dom(f.buffer_mut())).unwrap();
                        }
                        TickResult::Restart => {

                        }
                        TickResult::Exit => {
                            panic!("application exited");
                        }
                        TickResult::Command(command) => {
                            self.runtime
                                .handle_terminal_command(command,&mut self.terminal)
                                .await.unwrap();
                        }
                        TickResult::Continue => {}
                    }
                },
                _ = tokio::time::sleep(Duration::from_millis(10)) => {}
            }
            if f(self.buffer(), self) {
                return Ok(());
            }
            if Instant::now().duration_since(start) > timeout {
                return Err(self.buffer().clone());
            }
        }
    }

    pub async fn exit(mut self) {
        self.event_tx
            .send(Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
            )))
            .unwrap();

        let start = Instant::now();
        loop {
            tokio::select! {
                tick_result = self.runtime.tick() => {
                    if matches!(tick_result, TickResult::Exit) {
                        return;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(3) - (Instant::now() - start)) => {
                    panic!("application failed to exit");
                }
            }
        }
    }

    pub fn find_nth_position_of_text(&self, text: impl AsRef<str>, nth: usize) -> Option<Rect> {
        let text = text.as_ref();
        let view = self.buffer().terminal_view();
        let lines = view.split('\n');

        for (i, line) in lines.enumerate() {
            if let Some((col, _)) = line.match_indices(text).nth(nth) {
                return Some(Rect {
                    x: line[..col].width() as u16,
                    y: i as u16,
                    width: text.width() as u16,
                    height: 1,
                });
            }
        }
        None
    }

    pub fn get_nth_position_of_text(&self, text: impl AsRef<str>, nth: usize) -> Rect {
        self.find_nth_position_of_text(text, nth).unwrap()
    }

    pub fn find_position_of_text(&self, text: impl AsRef<str>) -> Option<Rect> {
        self.find_nth_position_of_text(text, 0)
    }

    pub fn get_position_of_text(&self, text: impl AsRef<str>) -> Rect {
        self.find_position_of_text(text).unwrap()
    }
}
