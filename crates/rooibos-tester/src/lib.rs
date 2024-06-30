use std::fmt::Write;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Terminal;
use rooibos_dom::{focus_next, render_dom, Render};
use rooibos_runtime::backend::test::TestBackend;
use rooibos_runtime::wasm_compat::{Lazy, RwLock};
use rooibos_runtime::{once, Runtime, RuntimeSettings, TickResult};

once! {
    static DEFAULT_TIMEOUT: Lazy<RwLock<Duration>> =
        Lazy::new(move || RwLock::new(Duration::from_secs(1)));
}

pub trait TerminalView {
    fn terminal_view(&self) -> String;
    fn slice(&self, rect: Rect) -> Self;
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

    fn slice(&self, rect: Rect) -> Self {
        let mut buf = Buffer::empty(rect);
        for row in rect.y..rect.y + rect.height {
            for col in rect.x..rect.x + rect.width {
                let cur = self.get(col, row);
                let new = buf.get_mut(col, row);

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
        let mut runtime = Runtime::initialize(runtime_settings, TestBackend::new(width, height), f);
        let mut terminal = runtime.setup_terminal().unwrap();
        terminal.draw(|f| render_dom(f.buffer_mut())).unwrap();
        focus_next();

        Self { runtime, terminal }
    }

    pub fn terminal(&self) -> &Terminal<ratatui::backend::TestBackend> {
        &self.terminal
    }

    pub async fn wait_for(&mut self, f: impl FnMut(&Buffer) -> bool) -> Result<(), Buffer> {
        DEFAULT_TIMEOUT
            .with(|t| t.with(|t| self.wait_for_timeout(f, *t)))
            .await
    }

    pub async fn wait_for_timeout(
        &mut self,
        mut f: impl FnMut(&Buffer) -> bool,
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
                            return Ok(());
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
            if f(self.terminal.backend().buffer()) {
                return Ok(());
            }
            if Instant::now().duration_since(start) > timeout {
                return Err(self.terminal.backend().buffer().clone());
            }
        }
    }
}
