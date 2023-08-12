use std::{
    error::Error,
    io::{stdout, Stdout},
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Block,
    Frame, Terminal,
};
use rooibos::{
    reactive::{create_signal, Scope, SignalGet, SignalUpdate},
    run_system, use_event_context, EventHandler,
};
use tui_rsx::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let (_handle, err) = run_system(run);
    err
}

#[tokio::main]
async fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();

    let handler = EventHandler::initialize(cx, terminal);

    let mut v = mount! { cx,
        <column>
            <Counter length=5/>
        </column>
    };
    handler.render(move |terminal| {
        terminal
            .draw(|f| {
                v.view(f, f.size());
            })
            .unwrap();
    });

    let mut terminal = handler.run().await;
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )
    .unwrap();
    terminal.show_cursor().unwrap();
    Ok(())
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
