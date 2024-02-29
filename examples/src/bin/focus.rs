use std::cell::RefCell;
use std::error::Error;
use std::fmt::format;
use std::io::{stdout, Stdout};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::Frame;
use rooibos::dom::prelude::*;
use rooibos::dom::{
    block, col, component, mount, print_dom, render_dom, row, view, BlockProps, Component,
    DocumentFragment, DomNode, IntoView, Mountable, ViewFragment,
};
use rooibos::reactive::{create_runtime, on_cleanup, RwSignal, SignalGet, SignalUpdate};
use rooibos::runtime::{create_key_effect, use_focus, Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = create_runtime();
    let mut rt = Runtime::initialize();
    let mut terminal = setup_terminal()?;
    mount(|| view!(<App/>));
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

#[component]
fn App() -> impl IntoView {
    // let focus_manager = use_focus_manager();

    create_key_effect(move |event| {
        if event.code == KeyCode::Up {
            focus_prev();
        }
        if event.code == KeyCode::Down {
            focus_next();
        }
    });

    view! {
        <Row>
            <Column v:percentage=50>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 1"/>
                </Row>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 2"/>
                </Row>
            </Column>
            <Column v:percentage=50>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 3"/>
                </Row>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 4"/>
                </Row>
            </Column>
        </Row>
    }
}

#[component]
fn FocusBlock(#[prop(into)] title: &'static str) -> impl IntoView {
    let (id, focused) = use_focus();

    view! {
        <Paragraph v:id=id block=prop!(<Block/>)>
            {format!("{title} - focused: {}", focused.get())}
        </Paragraph>
    }
}
