use std::error::Error;
use std::io::stdout;

use crossterm::event::{KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use rooibos::prelude::*;
use rooibos::reactive::{create_signal, Scope, SignalGet, SignalUpdate};
use rooibos::runtime::{run_system, use_event_context, EventHandler};
use tui_tree_widget::{Tree, TreeItem, TreeState};

mod prelude {
    use rooibos::prelude::*;
    use rooibos::rsx::{impl_stateful_render, make_builder};

    #[make_builder(suffix = "Demo")]
    pub trait DemoMakeBuilder {}

    impl_stateful_render!(DemoStatefulRender, visibility=pub);
}
use prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    run_system(run)
}

#[tokio::main]
async fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let handler = EventHandler::initialize(cx, terminal);

    handler.render(mount! { cx,
        <App/>
    });

    let mut terminal = handler.run().await;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    Ok(())
}

impl_stateful_widget!(Tree, generics=<'a>, stateful_render=DemoStatefulRender);
impl_widget!(Tree, generics=<'a>, make_builder=DemoMakeBuilder);

#[component]
fn App<B: Backend>(cx: Scope) -> impl View<B> {
    let state = create_signal(cx, TreeState::default());
    let tree = create_signal(
        cx,
        vec![
            TreeItem::new_leaf("a"),
            TreeItem::new(
                "b",
                vec![
                    TreeItem::new_leaf("c"),
                    TreeItem::new("d", vec![TreeItem::new_leaf("e"), TreeItem::new_leaf("f")]),
                    TreeItem::new_leaf("g"),
                ],
            ),
            TreeItem::new_leaf("h"),
        ],
    );

    let context = use_event_context(cx);
    context.create_key_effect(cx, move |event| {
        if event.kind == KeyEventKind::Press {
            match event.code {
                KeyCode::Char('\n' | ' ') => {
                    state.update(|mut s| {
                        s.toggle_selected();
                        s
                    });
                }
                KeyCode::Left => state.update(|mut s| {
                    s.key_left();
                    s
                }),
                KeyCode::Right => state.update(|mut s| {
                    s.key_right();
                    s
                }),
                KeyCode::Down => state.update(|mut s| {
                    s.key_down(&tree.get());
                    s
                }),
                KeyCode::Up => state.update(|mut s| {
                    s.key_up(&tree.get());
                    s
                }),
                KeyCode::Home => state.update(|mut s| {
                    s.select_first();
                    s
                }),
                KeyCode::End => state.update(|mut s| {
                    s.select_last(&tree.get());
                    s
                }),
                _ => {}
            }
        }
    });

    move || {
        view! { cx,
            <StatefulTree
                block=prop!(<Block borders=Borders::ALL title="Tree Widget"/>)
                highlight_style=prop!(<Style black on_green bold/>)
                v:state=state.get()
            >
                {tree.get()}
            </StatefulTree>
        }
    }
}
