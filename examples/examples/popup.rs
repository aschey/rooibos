use rooibos::dom::layout::{align_items, clear, justify_content, position, show};
use rooibos::dom::{
    col, height, line, max_height, max_width, row, wgt, width, KeyCode, KeyEvent, Render,
};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::Update;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::tui::widgets::{Block, Paragraph};
use taffy::{AlignItems, JustifyContent, Position};
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let show_popup = RwSignal::new(true);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            show_popup.update(|p| *p = !*p);
        }
    };

    col![
        props(max_width!(50.), max_height!(20.)),
        wgt!(
            Paragraph::new(vec![
                line!("text1"),
                line!("text2"),
                line!("text3"),
                line!("text4")
            ])
            .block(Block::bordered())
        ),
        row![
            props(
                width!(100.%),
                height!(100.%),
                position(Position::Absolute),
                align_items(AlignItems::Center),
                justify_content(JustifyContent::Center),
                show(show_popup)
            ),
            wgt!(
                props(clear(true), width!(25.), height!(5.)),
                Paragraph::new("popup text").block(Block::bordered())
            )
            .on_key_down(key_down)
        ],
    ]
}
