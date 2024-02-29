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
use rooibos::dom::{
    component, mount, prop, render_dom, view, Component, DomWidget, IntoView, Widget,
};
use rooibos::reactive::create_runtime;
use rooibos::runtime::{Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
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
fn App() -> impl IntoView {
    view! {
        <MyCustomWidget
            block=prop!(<Block title="custom widget"/>)
            text="widget text"
        />
    }
}

#[derive(Default, Clone, Widget)]
#[make_builder_trait(name=MakeBuilder)]
struct MyCustomWidget<'a> {
    block: Option<Block<'a>>,
    text: Text<'a>,
}

impl<'a> MyCustomWidget<'a> {
    fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn text(mut self, text: impl Into<Text<'a>>) -> Self {
        self.text = text.into();
        self
    }
}

impl<'a> Widget for MyCustomWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut tabs = Tabs::new(vec!["Tab1", "Tab2", "Tab3"]);
        if let Some(block) = self.block {
            tabs = tabs.block(block);
        }
        let paragraph = Paragraph::new(self.text);
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);
        tabs.render(chunks[0], buf);
        paragraph.render(chunks[1], buf);
    }
}
