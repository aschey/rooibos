use std::error::Error;

use rooibos::components::Show;
use rooibos::dom::layout::{align_items, justify_content, position};
use rooibos::dom::{
    after_render, col, focus_id, height, line, row, wgt, width, KeyCode, NodeId, Render,
};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{before_exit, exit, ExitResult, Runtime};
use rooibos::tui::widgets::{Block, Paragraph};
use taffy::{AlignItems, JustifyContent, Position};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let show_popup = RwSignal::new(false);
    let quit_confirmed = RwSignal::new(false);

    before_exit(move || async move {
        if quit_confirmed.get() {
            return ExitResult::Exit;
        }
        show_popup.set(true);
        ExitResult::PreventExit
    });

    col![
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
            ),
            Show::new().render(show_popup, move || {
                let popup_id = NodeId::new_auto();
                {
                    after_render({
                        let popup_id = popup_id.clone();
                        move || {
                            focus_id(popup_id);
                        }
                    });
                    wgt!(
                        props(height!(3.), width!(40.)),
                        Paragraph::new("Are you sure you want to quit? [yN]")
                            .block(Block::bordered())
                    )
                    .id(popup_id)
                    .on_key_down(move |key_event, _| {
                        if key_event.code == KeyCode::Char('y') {
                            quit_confirmed.set(true);
                            exit();
                        } else {
                            show_popup.set(false);
                        }
                    })
                }
            })
        ],
    ]
}
