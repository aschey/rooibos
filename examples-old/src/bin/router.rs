use std::backtrace::Backtrace;
use std::error::Error;
use std::io::stdout;

use crossterm::event::{DisableMouseCapture, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rooibos_old::reactive::{create_signal, Scope, Signal, SignalGet, SignalUpdate};
use rooibos_old::rsx::prelude::*;
use rooibos_old::runtime::{run_system, use_event_context, EventHandler};

fn main() -> Result<(), Box<dyn Error>> {
    run_system(run)
}

#[tokio::main]
async fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    std::panic::set_hook(Box::new(|panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        let backtrace = Backtrace::capture();
        println!("{panic_info} {backtrace}");
    }));

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let handler = EventHandler::initialize(cx, terminal);

    handler.render(mount! { cx,
        <App/>
    });

    let mut terminal = handler.run().await;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[component]
fn App(cx: Scope) -> impl View {
    let child2_id = create_signal(cx, 0);
    move || {
        view! { cx,
            <Router initial="/">
                <Route path="/">
                    {move || view!(cx, <Child0/>)}
                </Route>
                <Route path="/child1">
                    {move || view!(cx, <Child1 child2_id=child2_id/>)}
                </Route>
                <Route path="/child2/:id">
                    {move || view!(cx, <Child2/>)}
                </Route>
            </Router>
        }
    }
}

#[component]
fn Child0(cx: Scope) -> impl View {
    let router = use_router(cx);
    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
            router.push("/child1?id=1");
        }
    });
    move || {
        view! { cx,
            <Paragraph>
                "child0"
            </Paragraph>
        }
    }
}

#[component]
fn Child1(cx: Scope, child2_id: Signal<i32>) -> impl View {
    let router = use_router(cx);
    let id = router.use_query(cx, "id");
    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
            router.push(format!("/child2/{}", child2_id.get()));
            child2_id.update(|id| id + 1);
        }
    });

    move || {
        view! { cx,
            <Paragraph>
                {format!("child1 id={}", id.get().unwrap())}
            </Paragraph>
        }
    }
}

#[component]
fn Child2(cx: Scope) -> impl View {
    let router = use_router(cx);
    let id = router.use_param(cx, "id");
    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
            router.pop();
        }
    });

    move || {
        view! { cx,
            <Paragraph>
                {format!("child2 id={}", id.get().unwrap())}
            </Paragraph>
        }
    }
}
