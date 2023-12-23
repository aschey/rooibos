use std::cell::RefCell;
use std::error::Error;
use std::fmt::format;
use std::io::{stdout, Stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use leptos_reactive::{create_runtime, RwSignal, SignalGet, SignalUpdate};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::Frame;
use rooibos_dom::{
    block, col, mount, render_dom, BlockProps, DocumentFragment, DomNode, Fragment, IntoView,
    Mountable,
};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

thread_local! {
    static KEY_HANDLERS: RefCell<Vec<Box<dyn Fn(String)>>> = RefCell::new(vec![]);
}

fn main() -> Result<()> {
    let _ = create_runtime();
    let mut terminal = setup_terminal()?;
    mount(|| counters());
    terminal.draw(|f: &mut Frame| {
        render_dom(f);
    })?;
    loop {
        let e = handle_events()?;
        if e == 0 {
            restore_terminal(terminal)?;
            return Ok(());
        }
        if e == 1 {
            terminal.draw(|f: &mut Frame| {
                render_dom(f);
            })?;
        }
    }
}

fn handle_events() -> Result<usize> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(0);
            }
            if let KeyCode::Char(c) = key.code {
                KEY_HANDLERS.with(|h| h.borrow().iter().for_each(|h| (h)(c.to_string())));
                return Ok(1);
            }
        }
    }
    Ok(2)
}

fn setup_terminal() -> Result<Terminal> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn counter(initial_value: i32, step: u32) -> impl IntoView {
    let count = RwSignal::new(Count::new(initial_value, step));

    KEY_HANDLERS.with(|h| {
        h.borrow_mut().push(Box::new(move |key| {
            count.update(Count::increase);
        }))
    });

    block(move || BlockProps::default().title(format!("count: {}", count.get().value())))
}

fn counters() -> impl IntoView {
    col()
        .child(col().constraint(Constraint::Length(1)).child(counter(1, 1)))
        .child(col().constraint(Constraint::Length(1)).child(counter(2, 2)))
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
