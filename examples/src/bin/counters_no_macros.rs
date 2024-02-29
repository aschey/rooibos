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
    block, col, mount, print_dom, render_dom, row, BlockProps, CollectView, Component,
    DocumentFragment, DomNode, Fragment, IntoView, Mountable,
};
use rooibos::reactive::{create_runtime, on_cleanup, RwSignal, SignalGet, SignalUpdate};
use rooibos::runtime::{create_key_effect, Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut rt = Runtime::initialize();

    let mut terminal = setup_terminal()?;
    mount(counters);
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

fn counter(initial_value: i32, step: u32) -> impl IntoView {
    let count = RwSignal::new(Count::new(initial_value, step));

    create_key_effect(move |event| {
        if event.code == KeyCode::Enter {
            count.update(Count::increase);
        }
    });

    block(move || BlockProps::default().title(format!("count: {}", count.get().value())))
}

fn counters() -> impl IntoView {
    let count = RwSignal::new(Count::new(1, 1));

    create_key_effect(move |event| {
        if event.code == KeyCode::Char('i') {
            count.update(Count::increase);
        }
        if event.code == KeyCode::Char('d') {
            count.update(Count::decrease);
        }
    });

    col().child(move || {
        (1..count.get().value() + 1)
            .map(|i| {
                row()
                    .constraint(Constraint::Length(1))
                    .child(counter(i, i as u32))
            })
            .collect_view()
    })
}

#[derive(Debug, Clone)]
pub struct Count {
    value: i32,
    step: i32,
}

impl Count {
    pub fn new(value: i32, step: u32) -> Self {
        Count {
            value,
            step: step as i32,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn increase(&mut self) {
        self.value += self.step;
    }

    pub fn decrease(&mut self) {
        self.value += -self.step;
    }

    pub fn clear(&mut self) {
        self.value = 0;
    }
}
