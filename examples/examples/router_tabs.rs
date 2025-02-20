use std::process::ExitCode;

use rooibos::components::{Button, KeyedWrappingList, Tab, TabView};
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::layout::chars;
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, text};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{col, row};
use rooibos::router::{Route, RouteFromStatic, Router, use_router};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::Block;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
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
            .on_key_down(
                [
                    key(keys::LEFT, {
                        let tabs = tabs.clone();
                        move |_, _| {
                            if let Some(prev) = tabs.prev_item(&current_route.get()) {
                                router.push(Tabs::new(prev.get_value()));
                            }
                        }
                    }),
                    key(keys::RIGHT, {
                        let tabs = tabs.clone();
                        move |_, _| {
                            if let Some(next) = tabs.next_item(&current_route.get()) {
                                router.push(Tabs::new(next.get_value()));
                            }
                        }
                    })
                ]
                .bind()
            )
            .render(current_route, tabs),
        Button::new()
            .width(chars(14.))
            .height(chars(3.))
            .on_click(move || {
                router.back();
            })
            .enabled(router.can_go_back())
            .render(text!("Previous")),
        Button::new()
            .width(chars(14.))
            .height(chars(3.))
            .on_click(move || {
                router.forward();
            })
            .enabled(router.can_go_forward())
            .render(text!("Next"))
    ]
}
