use std::error::Error;

use rooibos::dom::{focus_next, mount, stateful_widget, KeyCode, KeyEvent, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::run;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::Block;
use tui_tree_widget::{Tree, TreeItem, TreeState};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    run().await?;
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

    Effect::new(move |_| {
        focus_next();
    });

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
            s.key_down(&tree.get());
        }),
        KeyCode::Up => state.update(|s| {
            s.key_up(&tree.get());
        }),
        KeyCode::Home => state.update(|s| {
            s.select_first(&tree.get());
        }),
        KeyCode::End => state.update(|s| {
            s.select_last(&tree.get());
        }),
        _ => {}
    };

    stateful_widget!(
        Tree::new(tree.get())
            .unwrap()
            .block(Block::bordered().title("Tree Widget"))
            .highlight_style(Style::default().black().on_green().bold()),
        state.get()
    )
    .on_key_down(key_down)
    .focusable(true)
}
