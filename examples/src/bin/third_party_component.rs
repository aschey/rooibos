use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{run, use_keypress};
use tui_textarea::TextArea;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(|| view!(<TextView/>));
    run().await?;
    Ok(())
}

#[component]
fn TextView() -> impl Render {
    let mut text_area = TextArea::default();
    text_area.set_block(prop!(<Block borders=Borders::ALL title="Example"/>));
    let text_area = RwSignal::new(text_area);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            text_area.update(|mut t| {
                let signal: crossterm::event::KeyEvent = term_signal.into();
                t.input(signal);
            });
        }
    });

    view! {
        <TextAreaWidget text_area=text_area/>
    }
}

#[component]
fn TextAreaWidget(text_area: RwSignal<TextArea<'static>>) -> impl Render {
    DomWidget::new("TextArea", move || {
        let widget = text_area.get();
        move |f: &mut Frame, area: Rect| {
            f.render_widget(widget.widget(), area);
        }
    })
}
