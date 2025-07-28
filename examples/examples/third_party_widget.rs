use std::hash::Hash;
use std::process::ExitCode;

use rooibos::reactive::KeyCode;
use rooibos::reactive::dom::events::KeyEventProps;
use rooibos::reactive::dom::layout::pct;
use rooibos::reactive::dom::widgets::{Role, WidgetRole};
use rooibos::reactive::dom::{DomWidget, MeasureNode, Render, RenderNode};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Track, Update, UpdateUntracked};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::{Style, Stylize};
use rooibos::tui::Frame;
use rooibos::tui::layout::Rect;
use rooibos::tui::widgets::{Block, StatefulWidget};
use tui_tree_widget::{Tree, TreeItem, TreeState};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let runtime = Runtime::initialize(DefaultBackend::auto());
    runtime.run(app).await
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

    DomWidget::new(move || {
        let tree = tree.get();
        state.track();
        RenderTree { tree, state }
    })
    .width(pct(100))
    .height(pct(100))
    .on_key_down(key_down)
}

struct RenderTree<T> {
    tree: Vec<TreeItem<'static, T>>,
    state: RwSignal<TreeState<T>>,
}

impl<T> RenderNode for RenderTree<T>
where
    T: Clone + PartialEq + Eq + Hash + Send + Sync + 'static,
{
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        self.state.update_untracked(|s| {
            Tree::new(&self.tree)
                .unwrap()
                .block(Block::bordered().title("Tree Widget"))
                .highlight_style(Style::default().black().on_green().bold().into())
                .render(area, frame.buffer_mut(), s);
        })
    }
}

impl<T> WidgetRole for RenderTree<T> {
    fn widget_role() -> Option<Role> {
        None
    }
}

impl<T> MeasureNode for RenderTree<T> {
    fn measure(
        &self,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
        _style: &taffy::Style,
    ) -> taffy::Size<f32> {
        taffy::Size::zero()
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        taffy::Size::zero()
    }
}
