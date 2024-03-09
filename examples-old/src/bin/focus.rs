use std::error::Error;
use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
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
    let focus_manager = use_focus_manager();

    let context = use_event_context(cx);

    context.create_key_effect(cx, move |event| {
        if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
            focus_manager.focus_next();
        }
    });

    move || {
        view! { cx,
            <FocusScope>
                <Col>
                    <FocusBlock v:percentage=50 v:focusable=true title="item 1"/>
                    <FocusBlock v:percentage=50 v:focusable=true title="item 2"/>
                </Col>
            </FocusScope>
        }
    }
}

#[component]
fn FocusBlock(cx: Scope, #[prop(into)] title: String) -> impl View {
    let focused = use_focus(cx);

    move || {
        view! { cx,
            <Paragraph block=prop!(<Block/>)>
                {format!("{title} - focused: {}", focused.get())}
            </Paragraph>
        }
    }
}
