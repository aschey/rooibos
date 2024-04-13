use std::cell::RefCell;
use std::error::Error;
use std::io::{stdout, Stdout};
use std::sync::atomic::Ordering;
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Buffer;
use ratatui::Frame;
use rooibos::dom::prelude::*;
use rooibos::dom::{component, mount, prop, render_dom, view, Widget};
use rooibos::runtime::{Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main(mut rt: Runtime) -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(|| view!(<App/>), rt.connect_update());
    // print_dom(&mut std::io::stdout(), false);
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

#[component]
fn App() -> impl Render {
    view! {
        <MyCustomWidget
            block=prop!(<Block title="custom widget"/>)
            text="widget text"
        />
    }
}

#[derive(Clone, Widget)]
#[make_builder_trait(name=MakeBuilder)]
#[render_ref(true)]
struct MyCustomWidget<'a> {
    paragraph: Paragraph<'a>,
    tabs: Tabs<'a>,
}

impl<'a> Default for MyCustomWidget<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MyCustomWidget<'a> {
    fn new() -> Self {
        Self {
            tabs: Tabs::new(vec!["Tab1", "Tab2", "Tab3"]),
            paragraph: Default::default(),
        }
    }
    fn block(mut self, block: Block<'a>) -> Self {
        self.tabs = self.tabs.block(block);
        self
    }

    fn text(mut self, text: impl Into<Text<'a>>) -> Self {
        self.paragraph = Paragraph::new(text);
        self
    }
}

impl<'a> Widget for MyCustomWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl<'a> WidgetRef for MyCustomWidget<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);
        self.tabs.render_ref(chunks[0], buf);
        self.paragraph.render_ref(chunks[1], buf);
    }
}
