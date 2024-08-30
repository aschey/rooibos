use std::sync::Arc;
use std::thread;

use futures::executor::block_on;
use rooibos::dom::{render_dom, wgt, KeyCode, KeyEvent, Render};
use rooibos::egui::backend::EguiBackend;
use rooibos::egui::eframe::{self, NativeOptions};
use rooibos::egui::egui::{Context, Event, Vec2};
use rooibos::egui::{egui, RataguiBackend};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{self, init_executor, run_with_executor, Runtime, TickResult};
use rooibos::tui::layout::Rect;
use rooibos::tui::Terminal;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task;

fn main() {
    runtime::execute(app_main)
}

fn app_main() {
    let options = NativeOptions::default();
    let (event_tx, event_rx) = mpsc::channel(32);
    let (term_tx, term_rx) = oneshot::channel();
    let (ctx_tx, ctx_rx) = oneshot::channel();
    std::thread::spawn(move || async_main(term_tx, event_rx, ctx_rx));
    let terminal = term_rx.blocking_recv().unwrap();
    // let rt = tokio::runtime::Runtime::new().unwrap();
    // let _guard = rt.enter();
    // init_executor();
    // let mut runtime = Runtime::initialize(EguiBackend::new(0, 0, rx), app);
    // let terminal = runtime.setup_terminal().unwrap();
    // let terminal = Arc::new(Mutex::new(terminal));
    eframe::run_native(
        "app",
        options,
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();
            ctx_tx.send(ctx).unwrap();
            Ok(Box::new(Content::new(terminal, event_tx)))
        }),
    )
    .unwrap();
}

#[rooibos::main]
async fn async_main(
    term_tx: oneshot::Sender<Arc<Mutex<Terminal<RataguiBackend>>>>,
    event_rx: mpsc::Receiver<Vec<Event>>,
    ctx_rx: oneshot::Receiver<Context>,
) {
    let mut runtime = Runtime::initialize(EguiBackend::new(100, 100, event_rx), app);
    let terminal = runtime.setup_terminal().unwrap();
    let terminal = Arc::new(Mutex::new(terminal));
    term_tx.send(terminal.clone()).unwrap();
    let ctx = ctx_rx.await.unwrap();

    terminal
        .lock()
        .await
        .draw(|f| render_dom(f.buffer_mut()))
        .unwrap();
    loop {
        let tick_result = runtime.tick().await.unwrap();
        let mut terminal = terminal.lock().await;
        match tick_result {
            TickResult::Redraw => {
                terminal.draw(|f| render_dom(f.buffer_mut())).unwrap();
            }
            TickResult::Restart => {
                *terminal = runtime.setup_terminal().unwrap();
                terminal.draw(|f| render_dom(f.buffer_mut())).unwrap();
            }
            TickResult::Exit => {
                if runtime.should_exit().await {
                    runtime.handle_exit(&mut terminal).await.unwrap();
                    return;
                }
            }
            TickResult::Command(command) => {
                runtime
                    .handle_terminal_command(command, &mut terminal)
                    .await
                    .unwrap();
            }
            TickResult::Continue => {}
        }
        ctx.request_repaint();
    }
}

struct Content {
    terminal: Arc<Mutex<Terminal<RataguiBackend>>>,
    tx: mpsc::Sender<Vec<Event>>,
    size: Vec2,
}

impl Content {
    fn new(terminal: Arc<Mutex<Terminal<RataguiBackend>>>, tx: mpsc::Sender<Vec<Event>>) -> Self {
        Self {
            terminal,
            tx,
            size: Vec2 { x: 0., y: 0. },
        }
    }
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            self.tx.try_send(i.events.clone()).unwrap();
        });
        let mut terminal = block_on(self.terminal.lock());
        let size = ctx.screen_rect().size();

        // if size != self.size {
        //     terminal
        //         .resize(Rect {
        //             x: 0,
        //             y: 0,
        //             width: (size.x / 16.) as u16,
        //             height: (size.y / 16.) as u16,
        //         })
        //         .unwrap();
        //     self.size = size;
        //     ctx.request_repaint();
        // }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(terminal.backend_mut());
        });
    }
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    wgt!(format!("count {}", count.get())).on_key_down(key_down)
}
