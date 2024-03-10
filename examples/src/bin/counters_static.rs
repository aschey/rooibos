use std::backtrace::Backtrace;
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
use ratatui::prelude::{Constraint, CrosstermBackend, Rect};
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

    std::panic::set_hook(Box::new(|panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        let backtrace = Backtrace::capture();
        println!("{panic_info} {backtrace}");
    }));

    mount(|| view!(<Counters/>));
    // print_dom(&mut std::io::stdout(), true);
    // Ok(())
    terminal.draw(|f: &mut Frame| {
        render_dom(f);
    })?;

    loop {
        if rt.tick().await == TickResult::Exit {
            restore_terminal(terminal)?;
            unmount();
            drop(rt);
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
fn Counter(id: u32, constraint: Constraint) -> impl IntoView {
    let (count, set_count) = create_signal(id);
    create_key_effect(move |event| {
        if event.code == KeyCode::Up {
            set_count.update(|c| *c += 1);
        }
        if event.code == KeyCode::Down {
            set_count.update(|c| *c -= 1);
        }
    });

    view! {
        <Block v:id=id.to_string() title=format!("count: {}", count.get()) v:constraint=constraint/>
    }
}

#[component]
fn Counters() -> impl IntoView {
    view! {
        <Col>
        {(0..5).map(|i| {
            view! {
                <Counter id=i constraint=Constraint::Length(2)/>
            }
        }).collect_view()}
        </Col>
    }
}
