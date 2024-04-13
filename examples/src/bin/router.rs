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
    block, col, component, mount, print_dom, render_dom, row, view, BlockProps, DocumentFragment,
    DomNode,
};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, GetUntracked, Update};
use rooibos::reactive::wrappers::read::Signal;
use rooibos::runtime::{key_effect, tick, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(|| view!(<App/>));

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
fn App() -> impl Render {
    let child2_id = RwSignal::new(0);

    view! {
        <Col>
            <Router>
                <Route path="/">
                    {move || view!(<Child0/>)}
                </Route>
                <Route path="/child1">
                    {move || view!(<Child1 child2_id=child2_id/>)}
                </Route>
                <Route path="/child2/{id}">
                    {move || view!(<Child2/>)}
                </Route>
        </Router>
    </Col>
    }
}

#[component]
fn Child0() -> impl Render {
    let router = use_router();

    key_effect(move |event| {
        if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
            router.push("/child1?id=1");
        }
    });
    view! {
        <Paragraph>
            "child0"
        </Paragraph>
    }
}

#[component]
fn Child1(child2_id: RwSignal<i32>) -> impl Render {
    let router = use_router();
    let id = router.use_query("id");

    key_effect(move |event| {
        if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
            router.push(format!("/child2/{}", child2_id.get_untracked()));
            child2_id.update(|id| *id += 1);
        }
    });

    view! {
        <Paragraph>
            {format!("child1 id={}", id.get().unwrap())}
        </Paragraph>
    }
}

#[component]
fn Child2() -> impl Render {
    let router = use_router();
    let id = router.use_param("id");

    key_effect(move |event| {
        if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
            router.pop();
        }
    });

    view! {
        <Paragraph>
            {format!("child2 id={}", id.get().unwrap())}
        </Paragraph>
    }
}
