use std::error::Error;

use rooibos::prelude::*;
use rooibos::runtime::{setup_terminal, tick, TickResult};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(|| view!(<App/>));

    loop {
        terminal
            .draw(|f: &mut Frame| {
                render_dom(f);
            })
            .unwrap();

        if tick().await == TickResult::Exit {
            return Ok(());
        }
    }
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
