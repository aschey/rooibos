use std::error::Error;
use std::sync::Arc;
use std::{mem, thread};

use accesskit::{Action, ActionRequest, TreeUpdate};
use accesskit_winit::Adapter;
use rooibos_dom::{NonblockingTerminal, focus_accesskit_id, process_accesskit_tree_updates};
use rooibos_reactive::dom::{Render, render_terminal};
use rooibos_reactive::spawn_local;
use rooibos_runtime::{Runtime, TickResult};
use tokio::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::Window;

pub use crate::backend::GuiBackend;

mod backend;
//mod lib2;

struct WindowState {
    window: Window,
    adapter: Adapter,
    pending_updates: Vec<TreeUpdate>,
    initial_tree_sent: bool,
}

struct GuiApp {
    state: Option<WindowState>,
    event_loop_proxy: EventLoopProxy<AppEvent>,
    event_tx: mpsc::Sender<ActionRequest>,
    terminal: Arc<tokio::sync::Mutex<NonblockingTerminal<GuiBackend>>>,
}

#[derive(Debug)]
pub enum AppEvent {
    Accesskit(accesskit_winit::Event),
    DomUpdates(Vec<TreeUpdate>),
    Draw,
}

impl From<accesskit_winit::Event> for AppEvent {
    fn from(value: accesskit_winit::Event) -> Self {
        Self::Accesskit(value)
    }
}

impl GuiApp {
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let window_attributes = Window::default_attributes().with_visible(false);

        let window = event_loop.create_window(window_attributes)?;
        let adapter =
            Adapter::with_event_loop_proxy(event_loop, &window, self.event_loop_proxy.clone());
        window.set_visible(true);

        self.state = Some(WindowState {
            window,
            adapter,
            pending_updates: Vec::new(),
            initial_tree_sent: false,
        });
        Ok(())
    }
}

impl ApplicationHandler<AppEvent> for GuiApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create initial window");
        if let Some(state) = self.state.as_ref() {
            state.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(window) => window,
            None => return,
        };
        state.adapter.process_event(&state.window, &event);
    }

    fn user_event(&mut self, _: &ActiveEventLoop, event: AppEvent) {
        let state = match &mut self.state {
            Some(window) => window,
            None => return,
        };
        match event {
            AppEvent::Accesskit(e) => match e.window_event {
                accesskit_winit::WindowEvent::InitialTreeRequested => {
                    let updates = mem::take(&mut state.pending_updates);
                    let first = updates.first().unwrap().clone();
                    let combined_update = TreeUpdate {
                        nodes: updates.into_iter().flat_map(|n| n.nodes).collect(),
                        focus: first.focus,
                        tree: first.tree.clone(),
                        tree_id: first.tree_id,
                    };
                    state.adapter.update_if_active(|| combined_update);
                }
                accesskit_winit::WindowEvent::ActionRequested(action_request) => {
                    self.event_tx.blocking_send(action_request).unwrap();
                }
                accesskit_winit::WindowEvent::AccessibilityDeactivated => {}
            },
            AppEvent::DomUpdates(e) => {
                if state.initial_tree_sent {
                    for event in e {
                        state.adapter.update_if_active(|| event);
                    }
                } else {
                    state.pending_updates.extend(e);
                }
            }
            AppEvent::Draw => {
                self.terminal
                    .blocking_lock()
                    .with_terminal_mut_blocking(|t| {
                        t.backend_mut();
                    });
            }
        }
    }
}

pub fn run<F>(f: F)
where
    F: FnOnce(AppHandle) + Send + 'static,
{
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    let (event_tx, event_rx) = mpsc::channel(32);
    let (term_tx, term_rx) = std::sync::mpsc::channel();
    let app_handle = AppHandle {
        proxy,
        event_rx,
        term_tx,
    };
    thread::spawn(move || f(app_handle));
    let terminal = term_rx.recv().unwrap();
    let mut app = GuiApp {
        state: None,
        event_loop_proxy: event_loop.create_proxy(),
        event_tx,
        terminal,
    };
    event_loop.run_app(&mut app).unwrap();
}

pub struct AppHandle {
    proxy: EventLoopProxy<AppEvent>,
    event_rx: mpsc::Receiver<ActionRequest>,
    term_tx: std::sync::mpsc::Sender<Arc<tokio::sync::Mutex<NonblockingTerminal<GuiBackend>>>>,
}

pub async fn run_async<F, M, P>(
    app_handle: AppHandle,
    mut runtime: Runtime<GuiBackend, P>,
    params: P,
    f: F,
) where
    F: FnOnce(P) -> M + 'static,
    M: Render,
    <M as Render>::DomState: 'static,
    P: 'static,
{
    let terminal = Arc::new(tokio::sync::Mutex::new(runtime.create_terminal().unwrap()));
    app_handle.term_tx.send(terminal.clone()).unwrap();
    runtime.mount(&mut *terminal.lock().await, params, f).await;
    let mut event_rx = app_handle.event_rx;
    spawn_local(async move {
        loop {
            while let Some(res) = event_rx.recv().await {
                match res.action {
                    Action::Focus => {
                        focus_accesskit_id(res.target_node);
                    }
                    Action::Click => {}
                    _ => {}
                }
            }
        }
    });
    loop {
        let tick_result = runtime.tick().await.unwrap();
        match tick_result {
            TickResult::Redraw => {
                render_terminal(&mut *terminal.lock().await).await.unwrap();
                process_accesskit_tree_updates(|t| {
                    app_handle
                        .proxy
                        .send_event(AppEvent::DomUpdates(t))
                        .unwrap();
                });
            }
            TickResult::Restart => {
                // terminal.join().await;
                // terminal = runtime.create_terminal().unwrap();
                // runtime.configure_terminal_events().await.unwrap();
                // render_terminal(&mut terminal).await.unwrap();
            }
            TickResult::Exit(payload) => {
                if runtime.should_exit(payload.clone()).await {
                    runtime
                        .handle_exit(&mut *terminal.lock().await)
                        .await
                        .unwrap();
                    return;
                }
            }
            TickResult::Command(command) => {
                runtime
                    .handle_terminal_command(command, &mut *terminal.lock().await)
                    .await
                    .unwrap();
            }
            TickResult::Continue => {}
        }
    }
}

// fn app() -> impl Render {
//     let (count, set_count) = signal(0);

//     let update_count = move || set_count.update(|c| *c += 1);

//     wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
//         .on_key_down(key(keys::ENTER, move |_, _| {
//             update_count();
//         }))
//         .on_click(move |_| update_count())
// }
