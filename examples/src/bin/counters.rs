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
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{key_effect, Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    rooibos::runtime::execute(async_main).unwrap();
    Ok(())
}

#[tokio::main]
async fn async_main() -> Result<()> {
    rooibos::runtime::init(async move {
        let mut rt = Runtime::initialize();

        let mut terminal = setup_terminal().unwrap();
        mount(|| view!(<Counters/>), rt.connect_update());
        // print_dom(&mut std::io::stdout(), false);
        terminal
            .draw(|f: &mut Frame| {
                render_dom(f);
            })
            .unwrap();

        loop {
            if rt.tick().await == TickResult::Exit {
                restore_terminal(terminal).unwrap();
                return;
            }
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();
        }
    })
    .await;
    Ok(())
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
fn Counter(id: u32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);
    key_effect(move |event| {
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
fn Counters() -> impl Render {
    let (n_counters, set_n_counters) = signal(2);
    key_effect(move |event| {
        if event.code == KeyCode::Enter {
            set_n_counters.update(|c| *c += 1);
        } else if event.code == KeyCode::Backspace {
            set_n_counters.update(|c| *c -= 1);
        }
    });

    view! {
        <Col>
            <ForEach
                each=move|| (0..n_counters.get())
                key=|i| *i
                children=move|i| {
                    view! {
                        <Counter id=i constraint=Constraint::Length(2)/>
                    }
                }
            />
        </Col>
    }
}
