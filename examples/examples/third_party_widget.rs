use std::any::type_name;
use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{DomWidget, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Track, Update, UpdateUntracked};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::buffer::Buffer;
use rooibos::tui::layout::Rect;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::{Block, StatefulWidget};
use tui_tree_widget::{Tree, TreeItem, TreeState};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
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

    let key_down = move |key_event: KeyEvent, _| match key_event.code {
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

    DomWidget::new(type_name::<Tree<&str>>(), move || {
        let tree = tree.get();
        state.track();
        move |rect: Rect, buf: &mut Buffer| {
            state.update_untracked(|s| {
                Tree::new(&tree)
                    .unwrap()
                    .block(Block::bordered().title("Tree Widget"))
                    .highlight_style(Style::default().black().on_green().bold())
                    .render(rect, buf, s);
            })
        }
    })
    .on_key_down(key_down)
}
