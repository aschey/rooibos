use rooibos::dom::{DomWidget, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::reactive::wrappers::read::MaybeSignal;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::tui::buffer::Buffer;
use rooibos::tui::layout::Rect;
use rooibos::tui::widgets::{Block, Widget};
use tui_textarea::TextArea;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let mut text_area_widget = TextArea::default();
    text_area_widget.set_block(Block::bordered().title("Example"));
    let text_area_widget = RwSignal::new(text_area_widget);

    let key_down = move |key_event: KeyEvent, _| {
        text_area_widget.update(|t| {
            if let Ok(event) =
                <KeyEvent as TryInto<crossterm::event::KeyEvent>>::try_into(key_event)
            {
                t.input(event);
            }
        });
    };

    text_area(text_area_widget).on_key_down(key_down)
}

fn text_area(text_area: impl Into<MaybeSignal<TextArea<'static>>>) -> DomWidget<()> {
    let text_area = text_area.into();
    DomWidget::new::<TextArea, _, _>(move || {
        let widget = text_area.get();
        move |area: Rect, buf: &mut Buffer| {
            widget.render(area, buf);
        }
    })
}
