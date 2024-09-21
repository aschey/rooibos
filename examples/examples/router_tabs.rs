use rooibos::components::{Button, KeyedWrappingList, Tab, TabView};
use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::layout::chars;
use rooibos::reactive::{Render, UpdateLayoutProps, col, line, mount, row, text};
use rooibos::router::{Route, RouteFromStatic, Router, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::Block;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

#[derive(Route)]
struct Tabs {
    id: String,
}

impl Tabs {
    const TAB1: &'static str = "tab1";
    const TAB2: &'static str = "tab2";
    const TAB3: &'static str = "tab3";

    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

fn app() -> impl Render {
    col![
        Router::new()
            .initial(Tabs::new(Tabs::TAB1))
            .routes([Route::new::<Tabs>(tabs)])
    ]
}

fn tabs() -> impl Render {
    let router = use_router();
    let count = RwSignal::new(0);
    let current_route = router.use_param(Tabs::ID);

    let tabs = KeyedWrappingList(vec![
        Tab::new(line!("Tab1"), Tabs::TAB1, move || "tab1"),
        Tab::new(line!("Tab2"), Tabs::TAB2, move || "tab2"),
        Tab::new(line!("Tab3"), Tabs::TAB3, move || "tab3"),
    ]);

    let on_key_down = {
        let tabs = tabs.clone();
        move |key_event: KeyEvent, _, _| match key_event.code {
            KeyCode::Left => {
                if let Some(prev) = tabs.prev_item(&current_route.get()) {
                    router.push(Tabs::new(prev.get_value()));
                }
            }
            KeyCode::Right => {
                if let Some(next) = tabs.next_item(&current_route.get()) {
                    router.push(Tabs::new(next.get_value()));
                }
            }
            _ => {}
        }
    };

    row![
        TabView::new()
            .header_height(chars(3.))
            .block(Block::bordered().title("Demo"))
            .highlight_style(Style::new().yellow())
            .fit(true)
            .on_title_click(move |_, tab| {
                count.update(|c| *c += 1);
                router.push(Tabs::new(tab));
            })
            .on_key_down(on_key_down)
            .render(current_route, tabs),
        Button::new()
            .width(chars(20.))
            .height(chars(3.))
            .on_click(move || {
                router.pop();
            })
            .render(text!("Previous"))
    ]
}
