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
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{tick, use_keypress, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(|| view!(<Counters/>));

    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();

    loop {
        if tick().await == TickResult::Exit {
            restore_terminal(terminal).unwrap();
            return Ok(());
        }
        terminal
            .draw(|f: &mut Frame| {
                render_dom(f);
            })
            .unwrap();
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
fn Counter(id: u32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                set_count.update(|c| *c += 1);
            }
            if term_signal.code == KeyCode::Down {
                set_count.update(|c| *c -= 1);
            }
        }
    });

    view! {
        <Block v:id=id.to_string() title=format!("count: {}", count.get()) v:constraint=constraint/>
    }
}

#[component]
fn Counters() -> impl Render {
    view! {
        <Col>
        {(0..5).map(|i| {
            view! {
                <Counter id=i constraint=Constraint::Length(2)/>
            }
        }).collect::<Vec<_>>()}
        </Col>
    }
}
