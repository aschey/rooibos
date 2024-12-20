use std::process::ExitCode;

use rooibos::reactive::KeyEvent;
use rooibos::reactive::dom::events::KeyEventProps;
use rooibos::reactive::dom::{DomWidget, Render, mount};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::graph::wrappers::read::MaybeSignal;
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::Frame;
use rooibos::tui::layout::Rect;
use rooibos::tui::widgets::{Block, Widget};
use tui_textarea::TextArea;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let mut text_area_widget = TextArea::default();
    text_area_widget.set_block(Block::bordered().title("Example"));
    let text_area_widget = RwSignal::new(text_area_widget);

    let key_down = move |props: KeyEventProps| {
        text_area_widget.update(|t| {
            if let Ok(event) =
                <KeyEvent as TryInto<crossterm::event::KeyEvent>>::try_into(props.event)
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
        move |area: Rect, frame: &mut Frame| {
            widget.render(area, frame.buffer_mut());
        }
    })
}
