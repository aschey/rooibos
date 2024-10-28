use std::process::ExitCode;

use rooibos::dom::{KeyCode, KeyEventProps};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Track, Update, UpdateUntracked};
use rooibos::reactive::{DomWidget, Render, mount};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::Frame;
use rooibos::tui::layout::Rect;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::{Block, StatefulWidget};
use tui_tree_widget::{Tree, TreeItem, TreeState};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let state = RwSignal::new(TreeState::default());
    let tree = RwSignal::new(vec![
        TreeItem::new_leaf("a", "a"),
        TreeItem::new("b", "b", vec![
            TreeItem::new_leaf("c", "c"),
            TreeItem::new("d", "d", vec![
                TreeItem::new_leaf("e", "e"),
                TreeItem::new_leaf("f", "f"),
            ])
            .unwrap(),
            TreeItem::new_leaf("g", "g"),
        ])
        .unwrap(),
        TreeItem::new_leaf("h", "h"),
    ]);

    let key_down = move |props: KeyEventProps| match props.event.code {
        KeyCode::Char('\n' | ' ') => {
            state.update(|s| {
                s.toggle_selected();
            });
        }
        KeyCode::Left => state.update(|s| {
            s.key_left();
        }),
        KeyCode::Right => state.update(|s| {
            s.key_right();
        }),
        KeyCode::Down => state.update(|s| {
            s.key_down();
        }),
        KeyCode::Up => state.update(|s| {
            s.key_up();
        }),
        KeyCode::Home => state.update(|s| {
            s.select_first();
        }),
        KeyCode::End => state.update(|s| {
            s.select_last();
        }),
        _ => {}
    };

    DomWidget::new::<Tree<&str>, _, _>(move || {
        let tree = tree.get();
        state.track();
        move |rect: Rect, frame: &mut Frame| {
            state.update_untracked(|s| {
                Tree::new(&tree)
                    .unwrap()
                    .block(Block::bordered().title("Tree Widget"))
                    .highlight_style(Style::default().black().on_green().bold())
                    .render(rect, frame.buffer_mut(), s);
            })
        }
    })
    .on_key_down(key_down)
}
