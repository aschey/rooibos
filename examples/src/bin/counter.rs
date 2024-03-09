use std::cell::{OnceCell, RefCell};
use std::error::Error;
use std::io::{stdout, Stdout};
use std::ops::ControlFlow;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;
use rooibos::dom::prelude::*;
use rooibos::reactive::{
    create_runtime, create_signal, ReadSignal, Signal, SignalGet, SignalSet, SignalUpdate,
};
use rooibos::runtime::{create_key_effect, Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut rt = Runtime::initialize();

    let mut terminal = setup_terminal()?;
    mount(|| view!(<Counter/>));
    // print_dom(&mut std::io::stdout(), false);
    terminal.draw(|f: &mut Frame| {
        render_dom(f);
    })?;

    loop {
        if rt.tick().await == TickResult::Exit {
            restore_terminal(terminal)?;
            return Ok(());
        }
        terminal.draw(|f: &mut Frame| {
            render_dom(f);
        })?;
    }
}

fn setup_terminal() -> Result<Terminal> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[component]
fn counter() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    create_key_effect(move |event| {
        if event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    });

    view! {
        <Col>
            {move || format!("count {}", count.get())}
        </Col>
    }
}
