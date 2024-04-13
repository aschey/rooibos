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
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{tick, use_keypress, TickResult};
use tui_tree_widget::{Tree, TreeItem, TreeState};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
use std::hash::Hash;
mod prelude {
    use rooibos::dom::make_builder;

    #[make_builder(suffix = "Demo")]
    pub(crate) trait DemoMakeBuilder {}
}
use prelude::*;

impl_widget!(
    Tree,
    visibility=pub,
    generics=<'a, Identifier: Clone + Eq + Hash + Default + 'static>,
    make_builder=DemoMakeBuilder,
    render_ref=false
);

impl_stateful_widget!(
    Tree,
    visibility=pub,
    generics=<'a, Identifier: Clone + Eq + Hash + 'static>,
    state_generics=<Identifier: Clone + Eq + Hash>,
    render_ref=false
);

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
    let state = RwSignal::new(TreeState::default());
    let tree = RwSignal::new(vec![
        TreeItem::new_leaf("a", "a"),
        TreeItem::new(
            "b",
            "b",
            vec![
                TreeItem::new_leaf("c", "c"),
                TreeItem::new(
                    "d",
                    "d",
                    vec![TreeItem::new_leaf("e", "e"), TreeItem::new_leaf("f", "f")],
                )
                .unwrap(),
                TreeItem::new_leaf("g", "g"),
            ],
        )
        .unwrap(),
        TreeItem::new_leaf("h", "h"),
    ]);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            match term_signal.code {
                KeyCode::Char('\n' | ' ') => {
                    state.update(|mut s| {
                        s.toggle_selected();
                    });
                }
                KeyCode::Left => state.update(|mut s| {
                    s.key_left();
                }),
                KeyCode::Right => state.update(|mut s| {
                    s.key_right();
                }),
                KeyCode::Down => state.update(|mut s| {
                    s.key_down(&tree.get());
                }),
                KeyCode::Up => state.update(|mut s| {
                    s.key_up(&tree.get());
                }),
                KeyCode::Home => state.update(|mut s| {
                    s.select_first(&tree.get());
                }),
                KeyCode::End => state.update(|mut s| {
                    s.select_last(&tree.get());
                }),
                _ => {}
            }
        }
    });

    view! {
        <StatefulTree
            unwrap
            block=prop!(<Block borders=Borders::ALL title="Tree Widget"/>)
            highlight_style=prop!(<Style black on_green bold/>)
            v:state=move || state.get()
        >
            {tree.get()}
        </StatefulTree>
    }
}
