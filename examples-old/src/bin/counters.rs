use std::error::Error;
use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::{Frame, Terminal};
use rooibos_old::reactive::{create_signal, Scope, SignalGet, SignalUpdate};
use rooibos_old::rsx::prelude::*;
use rooibos_old::runtime::{run_system, use_event_context, EventHandler};

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
fn Counters(cx: Scope) -> impl View {
    let n_counters = create_signal(cx, 1);
    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.kind == KeyEventKind::Press {
            match event.code {
                KeyCode::Char('a') => {
                    n_counters.update(|c| c + 1);
                }
                KeyCode::Char('r') => {
                    n_counters.update(|c| (c - 1).max(1));
                }
                _ => {}
            }
        }
    });

    move || {
        view! { cx,
            <Column>
                {(0..n_counters.get()).map(|i| {
                    view!(cx, <Counter v:key=i/>).into_boxed_view()
                }).collect::<Vec<_>>()}
            </Column>
        }
    }
}

#[component]
fn Counter(cx: Scope) -> impl View {
    let count = create_signal(cx, 0);
    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.kind == KeyEventKind::Press && event.code == KeyCode::Enter {
            count.update(|c| c + 1);
        }
    });

    move || {
        view! { cx,
            <Block title=format!("count {}",  count.get())/>
        }
    }
}
