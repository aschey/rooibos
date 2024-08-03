use std::error::Error;
use std::io::Stdout;

use rooibos::components::Popup;
use rooibos::dom::{
    clear, col, fill, line, overlay, props, widget_ref, Constrainable, KeyCode, KeyEvent, Render,
};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::Update;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::widgets::{Block, Paragraph};

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
    let show_popup = RwSignal::new(false);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            show_popup.update(|p| *p = !*p);
        }
    };

    overlay![
        widget_ref!(
            Paragraph::new(vec![
                line!("text1"),
                line!("text2"),
                line!("text3"),
                line!("text4")
            ])
            .block(Block::bordered())
        )
        .on_key_down(key_down),
        Popup::default()
            .percent_x(50)
            .percent_y(50)
            .render(show_popup, move || col![
                col![props!(fill(1));],
                clear![widget_ref!(
                    Paragraph::new("popup text").block(Block::bordered())
                )]
                .length(3),
                col![props!(fill(1));],
            ])
    ]
}
