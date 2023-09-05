use std::error::Error;
use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::{Frame, Terminal};
use rooibos::reactive::{create_signal, Scope, SignalGet, SignalUpdate};
use rooibos::rsx::prelude::*;
use rooibos::runtime::{run_system, use_event_context, EventHandler};

fn main() -> Result<(), Box<dyn Error>> {
    run_system(run)
}

#[tokio::main]
async fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let handler = EventHandler::initialize(cx, terminal);

    handler.render(mount! { cx,
        <Counters/>
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
fn Counters<B: Backend + 'static>(cx: Scope) -> impl rooibos::rsx::View<B> {
    let n_counters = create_signal(cx, 1);
    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.code == KeyCode::Char('a') {
            n_counters.update(|c| *c += 1);
        }
        if event.code == KeyCode::Char('r') {
            n_counters.update(|c| *c = (*c - 1).max(1));
        }
    });

    move || {
        view! { cx,
            <column>
                {(0..n_counters.get()).map(|i| {
                    view!(cx, <Counter key=i/>).into_boxed_view()
                }).collect::<Vec<_>>()}
            </column>
        }
    }
}

#[component]
fn Counter<B: Backend + 'static>(cx: Scope) -> impl View<B> {
    let count = create_signal(cx, 0);
    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.code == KeyCode::Enter {
            count.update(|c| *c += 1);
        }
    });

    move || {
        view! { cx,
            <block title=format!("count {}",  count.get())/>
        }
    }
}