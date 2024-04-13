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
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{key_effect, Runtime, TickResult};
use tui_textarea::TextArea;

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main(mut rt: Runtime) -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(|| view!(<TextView/>), rt.connect_update());

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
fn TextView() -> impl Render {
    let mut text_area = TextArea::default();
    text_area.set_block(prop!(<Block borders=Borders::ALL title="Example"/>));
    let text_area = RwSignal::new(text_area);

    key_effect(move |event| {
        text_area.update(|mut t| {
            t.input(event);
        });
    });
    view! {
        <TextAreaWidget text_area=text_area/>
    }
}

#[component]
fn TextAreaWidget(text_area: RwSignal<TextArea<'static>>) -> impl Render {
    DomWidget::new("TextArea", move || {
        let widget = text_area.get();
        move |f: &mut Frame, area: Rect| {
            f.render_widget(widget.widget(), area);
        }
    })
}
