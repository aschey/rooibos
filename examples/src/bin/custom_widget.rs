use std::error::Error;
use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Buffer;
use ratatui::Terminal;
use rooibos::reactive::Scope;
use rooibos::rsx::prelude::*;
use rooibos::runtime::{run_system, EventHandler};

fn main() -> Result<(), Box<dyn Error>> {
    run_system(run)
}

#[tokio::main]
async fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let handler = EventHandler::initialize(cx, terminal);

    handler.render(mount! { cx,
        <App/>
    });

    let mut terminal = handler.run().await;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[component]
fn App<B: Backend>(cx: Scope) -> impl View<B> {
    move || {
        view! { cx,
            <MyCustomWidget
                block=prop!(<Block title="custom widget"/>)
                text="widget text"
            />
        }
    }
}

#[derive(Default, Clone, Widget)]
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
