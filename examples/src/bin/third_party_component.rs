use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{DomWidget, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::reactive::wrappers::read::MaybeSignal;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{start, RuntimeSettings};
use rooibos::tui::layout::Rect;
use rooibos::tui::widgets::Block;
use rooibos::tui::Frame;
use tui_textarea::TextArea;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let handle = start(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    handle.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let mut text_area_widget = TextArea::default();
    text_area_widget.set_block(Block::bordered().title("Example"));
    let text_area_widget = RwSignal::new(text_area_widget);

    let key_down = move |key_event: KeyEvent, _| {
        text_area_widget.update(|t| {
            let signal: crossterm::event::KeyEvent = key_event.into();
            t.input(signal);
        });
    };

    text_area(text_area_widget)
        .focusable(true)
        .on_key_down(key_down)
}

fn text_area(text_area: impl Into<MaybeSignal<TextArea<'static>>>) -> DomWidget {
    let text_area = text_area.into();
    DomWidget::new("TextArea", move || {
        let widget = text_area.get();
        move |f: &mut Frame, area: Rect| {
            f.render_widget(widget.widget(), area);
        }
    })
}
