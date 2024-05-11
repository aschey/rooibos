use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::run;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    run().await?;
    Ok(())
}

fn app() -> impl Render {
    let show_popup = RwSignal::new(false);

    Effect::new(move |_| {
        focus_next();
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            show_popup.update(|p| *p = !*p);
        }
    };

    overlay![
        widget_ref!(
            Paragraph::new(vec![
                Line::new("text1"),
                Line::new("text2"),
                Line::new("text3"),
                Line::new("text4")
            ])
            .block(Block::bordered())
        )
        .focusable(true)
        .on_key_down(key_down),
        Popup::default().percent_x(50).percent_y(50).render(
            move || show_popup.get(),
            move || widget_ref!(Paragraph::new("popup text").block(Block::bordered()))
                .constraint(Constraint::Length(3))
        )
    ]
}
