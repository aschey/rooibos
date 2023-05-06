use std::io::{stdout, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use leptos_reactive::{create_effect, create_signal, Scope, SignalGet, SignalUpdate};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Block,
    Frame, Terminal,
};
use rooibos::{run_system, use_event_provider, Event, EventHandler};
use tui_rsx::prelude::*;

#[derive(Clone, PartialEq, Eq)]
enum CustomEvent {
    Increment,
}

fn main() {
    let _scope = run_system(run);
}

#[tokio::main]
async fn run(cx: Scope) {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();

    let handler = EventHandler::initialize(cx, terminal);

    let counter = create_counter(cx);
    let label = create_label(cx);
    let test = create_test(cx);

    handler.render(move |terminal| {
        terminal
            .draw(|f| {
                let v = view! {
                    <Column>
                        <counter length=5/>
                        <test length=2 />
                        <block length=3 style=prop!(<style bg=Color::Black/>)/>
                        <label min=0/>
                    </Column>
                };
                v(f, f.size());
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
}

fn create_test(cx: Scope) -> impl Fn(&mut Frame<CrosstermBackend<Stdout>>, Rect) {
    let counter = create_counter(cx);
    let label = create_label(cx);
    view! { move
        <Row>
            <counter length=10/>
            <label min=0/>
        </Row>
    }
}

fn create_counter(cx: Scope) -> impl Fn(&mut Frame<CrosstermBackend<Stdout>>, Rect) {
    let (count, set_count) = create_signal(cx, 0);
    let sig = use_event_provider(cx).create_event_signal();
    // let sig = provider.create_event_signal();

    create_effect(cx, move |_| match sig() {
        Some(Event::TermEvent(crossterm::event::Event::Key(KeyEvent {
            code: KeyCode::Enter,
            ..
        }))) => {
            set_count.update(|c| *c += 1);
        }
        // Some(Event::Custom(CustomEvent::Increment)) => {
        //     set_count.update(|c| *c += 1);
        // }
        _ => {}
    });

    view! {
        move <block title=format!("count {}", count.get())/>
    }
}

fn create_label(cx: Scope) -> impl Fn(&mut Frame<CrosstermBackend<Stdout>>, Rect) {
    view! { <block title="test"/> }
}
