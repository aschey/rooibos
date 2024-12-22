use std::process::ExitCode;

use rooibos::reactive::KeyEvent;
use rooibos::reactive::dom::events::KeyEventProps;
use rooibos::reactive::dom::layout::chars;
use rooibos::reactive::dom::{DomWidget, MeasureNode, Render, RenderNode};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{GetUntracked as _, Track as _, Update as _, With as _};
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
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run(app).await
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

    text_area(text_area_widget)
        .on_key_down(key_down)
        .min_width(chars(9.))
}

fn text_area(text_area: RwSignal<TextArea<'static>>) -> DomWidget<()> {
    DomWidget::new::<TextArea, _>(move || {
        text_area.track();
        RenderInput { text_area }
    })
}

struct RenderInput {
    text_area: RwSignal<TextArea<'static>>,
}

impl RenderNode for RenderInput {
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        self.text_area.with(|t| t.render(area, frame.buffer_mut()));
    }
}

impl MeasureNode for RenderInput {
    fn measure(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        style: &taffy::Style,
    ) -> taffy::Size<f32> {
        let text = self.text_area.get_untracked();
        let lines = text.lines();
        let max_len = lines
            .iter()
            .map(|l| l.measure(known_dimensions, available_space, style).width)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.);

        taffy::Size {
            // +1 for cursor, +2 for borders
            width: (text.placeholder_text().len() as f32).max(max_len) + 3.,
            height: lines.len() as f32 + 2.0,
        }
    }

    fn estimate_size(&self) -> taffy::Size<f32> {
        let text = self.text_area.get_untracked();
        let lines = text.lines();
        let max_len = lines
            .iter()
            .map(|l| l.estimate_size().width)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.);

        taffy::Size {
            // +1 for cursor, +2 for borders
            width: (text.placeholder_text().len() as f32).max(max_len) + 3.,
            height: lines.len() as f32 + 2.0,
        }
    }
}
