use std::error::Error;

use rooibos::components::Popup;
use rooibos::dom::{
    after_render, clear, col, fill, focus_id, line, overlay, widget_ref, Constrainable, KeyCode,
    NodeId, Render,
};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{before_exit, exit, ExitResult, Runtime};
use rooibos::tui::widgets::{Block, Paragraph};

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

    before_exit(move || {
        if quit_confirmed.get() {
            return ExitResult::Exit;
        }
        show_popup.set(true);
        ExitResult::PreventExit
    });

    overlay![
        widget_ref!(
            Paragraph::new(vec![
                line!("text1"),
                line!("text2"),
                line!("text3"),
                line!("text4")
            ])
            .block(Block::bordered())
        ),
        Popup::default()
            .percent_x(50)
            .percent_y(50)
            .render(show_popup, move || {
                let popup_id = NodeId::new_auto();
                {
                    after_render({
                        let popup_id = popup_id.clone();
                        move || {
                            focus_id(popup_id);
                        }
                    });
                }

                col![
                    col![props(fill(1))],
                    clear![
                        widget_ref!(
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
                    ]
                    .length(3),
                    col![props(fill(1))],
                ]
            })
    ]
}
