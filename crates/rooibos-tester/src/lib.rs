mod test_backend;

use std::fmt::Write;
use std::mem;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rooibos_dom::{
    DomNodeRepr, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    NodeTypeRepr, NonblockingTerminal, focus_next, render_terminal,
};
use rooibos_reactive::dom::Render;
#[cfg(feature = "runtime")]
use rooibos_runtime::wasm_compat::{self, Lazy, RwLock};
#[cfg(feature = "runtime")]
use rooibos_runtime::{Runtime, RuntimeSettings, TickResult};
pub use test_backend::*;
use tokio::sync::broadcast;
use unicode_width::UnicodeWidthStr;

#[cfg(feature = "runtime")]
wasm_compat::static_init! {
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
                let cell = self.cell((col, row)).unwrap();
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
        let Rect { width, height, .. } = self.area();
        for row in rect.y..(rect.y + rect.height).min(*height) {
            for col in rect.x..(rect.x + rect.width).min(*width) {
                let cur = self.cell((col, row)).unwrap();
                let new = buf.cell_mut((col - rect.x, row - rect.y)).unwrap();

                new.set_skip(cur.skip);
                new.set_style(cur.style());
                new.set_symbol(cur.symbol());
            }
        }
        buf
    }
}

pub struct TestHarness<P = ()> {
    #[cfg(feature = "runtime")]
    runtime: Runtime<TestBackend, P>,
    terminal: NonblockingTerminal<ratatui::backend::TestBackend>,
    event_tx: broadcast::Sender<Event>,
}

impl TestHarness<()> {
    #[cfg(feature = "runtime")]
    pub async fn new(width: u16, height: u16) -> Self {
        Self::new_with_settings(RuntimeSettings::default(), width, height).await
    }

    #[cfg(feature = "runtime")]
    pub async fn new_with_settings(
        runtime_settings: RuntimeSettings,
        width: u16,
        height: u16,
    ) -> Self {
        use rooibos_theme::{Color, ColorPalette, ColorScheme, SetTheme, TermProfile};

        let backend = TestBackend::new(width, height);
        let event_tx = backend.event_tx();
        let mut runtime = Runtime::initialize_with(runtime_settings, backend);
        TermProfile::TrueColor.set();
        ColorPalette {
            terminal_fg: Color::White,
            terminal_bg: Color::Black,
            color_scheme: ColorScheme::Dark,
        }
        .set();

        let terminal = runtime.create_terminal().unwrap();
        runtime.configure_terminal_events().await.unwrap();

        Self {
            runtime,
            terminal,
            event_tx,
        }
    }

    pub async fn from_terminal(
        mut terminal: NonblockingTerminal<ratatui::backend::TestBackend>,
        width: u16,
        height: u16,
    ) -> Self {
        let backend = TestBackend::new(width, height);
        let event_tx = backend.event_tx();
        render_terminal(&mut terminal).await.unwrap();
        focus_next();

        Self {
            terminal,
            event_tx,
            #[cfg(feature = "runtime")]
            runtime: rooibos_runtime::Runtime::initialize(backend),
        }
    }
}

impl<P> TestHarness<P>
where
    P: 'static,
{
    #[cfg(feature = "runtime")]
    pub fn set_default_timeout(timeout: Duration) {
        DEFAULT_TIMEOUT.with(|t| *t.write() = timeout);
    }

    pub async fn mount<F, M>(&mut self, params: P, f: F)
    where
        F: FnOnce(P) -> M + Send + 'static,
        M: Render,
        <M as Render>::DomState: 'static,
    {
        self.runtime.mount(&mut self.terminal, params, f).await;
        render_terminal(&mut self.terminal).await.unwrap();
        focus_next();
    }

    pub fn terminal(&self) -> &NonblockingTerminal<ratatui::backend::TestBackend> {
        &self.terminal
    }

    pub async fn buffer(&self) -> Buffer {
        self.terminal
            .with_terminal(|t| t.backend().buffer().clone())
            .await
    }

    pub async fn terminal_view(&self) -> String {
        self.buffer().await.terminal_view()
    }

    #[cfg(feature = "runtime")]
    pub async fn wait_for<F>(&mut self, f: F) -> Result<(), Buffer>
    where
        F: AsyncFnMut(&Self, &TickEvents) -> bool,
    {
        DEFAULT_TIMEOUT
            .with(|t| self.wait_for_timeout(f, *t.read()))
            .await
    }

    pub async fn find_by_text(
        &self,
        node: &DomNodeRepr,
        text: impl AsRef<str>,
    ) -> Option<DomNodeRepr> {
        let text = text.as_ref();
        let buf = self.buffer().await;
        node.find(|node| {
            node.node_type() == NodeTypeRepr::Widget
                && buf.slice(node.rect()).terminal_view().contains(text)
        })
    }

    pub async fn find_by_block_text(
        &self,
        node: &DomNodeRepr,
        text: impl AsRef<str>,
    ) -> Option<DomNodeRepr> {
        let text = text.as_ref();
        let buf = self.buffer().await;
        node.find(|node| {
            if let NodeTypeRepr::Layout { borders } = node.node_type()
                && borders.is_some()
            {
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
            false
        })
    }

    pub async fn get_by_text(&self, node: &DomNodeRepr, text: impl AsRef<str>) -> DomNodeRepr {
        if let Some(node) = self.find_by_text(node, text).await {
            node
        } else {
            panic!("{}", self.buffer().await.terminal_view());
        }
    }

    pub async fn get_by_block_text(
        &self,
        node: &DomNodeRepr,
        text: impl AsRef<str>,
    ) -> DomNodeRepr {
        if let Some(node) = self.find_by_block_text(node, text).await {
            node
        } else {
            panic!("{}", self.buffer().await.terminal_view());
        }
    }

    pub async fn find_all_by_text(
        &self,
        node: &DomNodeRepr,
        text: impl AsRef<str>,
    ) -> Vec<DomNodeRepr> {
        let text = text.as_ref();
        let buffer = self.buffer().await;
        node.find_all(|node| {
            node.node_type() == NodeTypeRepr::Widget
                && buffer.slice(node.rect()).terminal_view().contains(text)
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

    #[cfg(feature = "runtime")]
    pub async fn wait_for_timeout<F>(&mut self, mut f: F, timeout: Duration) -> Result<(), Buffer>
    where
        F: AsyncFnMut(&Self, &TickEvents) -> bool,
    {
        use rooibos_dom::render_terminal;

        let start = Instant::now();
        let mut last_tick_events = TickEvents::default();
        loop {
            tokio::select! {
                biased;

                tick_result = self.runtime.tick() => {
                    let tick_result = tick_result.unwrap();
                    match tick_result {
                        TickResult::Redraw => {
                            render_terminal(&mut self.terminal).await.unwrap();
                        }
                        TickResult::Restart => {
                            let mut new_term = self.runtime.create_terminal().unwrap();
                            mem::swap(&mut new_term, &mut self.terminal);
                            new_term.join().await;
                            self.runtime.configure_terminal_events().await.unwrap();
                            render_terminal(&mut self.terminal).await.unwrap();
                            last_tick_events.restart = true;
                        }
                        TickResult::Exit(_) => {
                            panic!("application exited");
                        }
                        TickResult::Command(command) => {
                            self.runtime
                                .handle_terminal_command(command,&mut self.terminal)
                                .await.unwrap();
                        }
                        TickResult::Continue => {}
                    }
                    // Don't check output until debounce timeout has completed
                    // This ensures we don't check the output against partial updates
                    continue;
                },
                _ = tokio::time::sleep(Duration::from_millis(10)) => {}
            }
            if f(self, &last_tick_events).await {
                // Ensure the screen is updated in case we're still in a debouncer timeout
                render_terminal(&mut self.terminal).await.unwrap();
                return Ok(());
            } else {
                last_tick_events = TickEvents::default();
            }
            if Instant::now().duration_since(start) > timeout {
                return Err(self.buffer().await.clone());
            }
        }
    }

    #[cfg(feature = "runtime")]
    pub async fn exit(mut self) {
        use rooibos_runtime::proc_exit;

        self.event_tx
            .send(Event::Key(
                KeyEvent::new(KeyCode::Char('c')).modifiers(KeyModifiers::CTRL),
            ))
            .unwrap();

        let start = Instant::now();
        loop {
            tokio::select! {
                tick_result = self.runtime.tick() => {
                    let tick_result = tick_result.unwrap();
                    if let TickResult::Exit(payload) = tick_result
                        && self.runtime.should_exit(payload.clone()).await {
                            assert_eq!(payload.code(), proc_exit::Code::SUCCESS);
                            self.runtime.handle_exit(&mut self.terminal).await.unwrap();
                            return;
                        }
                }
                _ = tokio::time::sleep(Duration::from_secs(3) - (Instant::now() - start)) => {
                    panic!("application failed to exit");
                }
            }
        }
    }

    pub async fn find_nth_position_of_text(
        &self,
        text: impl AsRef<str>,
        nth: usize,
    ) -> Option<Rect> {
        let text = text.as_ref();
        let view = self.buffer().await.terminal_view();
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

    pub async fn get_nth_position_of_text(&self, text: impl AsRef<str>, nth: usize) -> Rect {
        self.find_nth_position_of_text(text, nth).await.unwrap()
    }

    pub async fn find_position_of_text(&self, text: impl AsRef<str>) -> Option<Rect> {
        self.find_nth_position_of_text(text, 0).await
    }

    pub async fn get_position_of_text(&self, text: impl AsRef<str>) -> Rect {
        self.find_position_of_text(text).await.unwrap()
    }
}

#[derive(Clone, Default)]
pub struct TickEvents {
    pub restart: bool,
}
