use std::cell::RefCell;
use std::error::Error;
use std::fmt::format;
use std::io::{stdout, Stdout};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::Frame;
use rooibos::dom::{
    block, col, mount, print_dom, render_dom, row, BlockProps, DomNode, DomWidget, Render,
};
use rooibos::reactive::owner::Owner;
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{key_effect, Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main(mut rt: Runtime) -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(counter, rt.connect_update());
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();

    loop {
        if rt.tick().await == TickResult::Exit {
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

fn counter() -> impl Render {
    let (count, set_count) = signal(1);

    key_effect(move |event| {
        if event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    });

    block(move || {
        let c = count.get();
        return BlockProps::default().title(format!("count: {}", c));
    })
}
