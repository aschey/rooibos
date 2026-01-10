use std::process::ExitCode;

use rooibos::reactive::dom::events::KeyEventProps;
use rooibos::reactive::dom::widgets::{Role, WidgetRole};
use rooibos::reactive::dom::{DomWidget, MeasureNode, Render, RenderNode};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{GetUntracked as _, Track as _, Update as _, With as _};
use rooibos::reactive::{Event, KeyCode, KeyModifiers, Repeats, ScrollDirection};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::Frame;
use rooibos::tui::layout::Rect;
use rooibos::tui::widgets::{Block, Widget};
use tui_textarea::TextArea;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let runtime = Runtime::initialize(DefaultBackend::auto().await?);
    runtime.run(|_| app()).await
}

fn app() -> impl Render {
    let mut text_area_widget = TextArea::default();
    text_area_widget.set_block(Block::bordered().title("Example"));
    let text_area_widget = RwSignal::new(text_area_widget);

    let key_down = move |props: KeyEventProps| {
        text_area_widget.update(|t| {
            if let Some(event) = to_input(Event::Key(props.event)) {
                t.input(event);
            }
        });
    };

    text_area(text_area_widget)
        .on_key_down(key_down)
        .min_width(9)
}

fn text_area(text_area: RwSignal<TextArea<'static>>) -> DomWidget<()> {
    DomWidget::new(move || {
        text_area.track();
        RenderInput { text_area }
    })
}

fn to_input(event: Event) -> Option<tui_textarea::Input> {
    let (key, modifiers) = to_key(event)?;
    Some(tui_textarea::Input {
        key,
        shift: modifiers.intersects(KeyModifiers::SHIFT),
        ctrl: modifiers.intersects(KeyModifiers::CTRL),
        alt: modifiers.intersects(KeyModifiers::ALT),
    })
}

fn to_key(event: Event) -> Option<(tui_textarea::Key, KeyModifiers)> {
    if let Some((mouse_event, direction)) = event.as_mouse_scroll() {
        match direction {
            ScrollDirection::Up => {
                return Some((tui_textarea::Key::MouseScrollUp, mouse_event.modifiers));
            }
            ScrollDirection::Down => {
                return Some((tui_textarea::Key::MouseScrollDown, mouse_event.modifiers));
            }
            _ => {}
        }
    }

    let key_event = event.as_key_press(Repeats::Include)?;

    let key_code = match key_event.code {
        KeyCode::Backspace => tui_textarea::Key::Backspace,
        KeyCode::Enter => tui_textarea::Key::Enter,
        KeyCode::Left => tui_textarea::Key::Left,
        KeyCode::Right => tui_textarea::Key::Right,
        KeyCode::Up => tui_textarea::Key::Up,
        KeyCode::Down => tui_textarea::Key::Down,
        KeyCode::Home => tui_textarea::Key::Home,
        KeyCode::End => tui_textarea::Key::End,
        KeyCode::PageUp => tui_textarea::Key::PageUp,
        KeyCode::PageDown => tui_textarea::Key::PageDown,
        KeyCode::Tab => tui_textarea::Key::Tab,
        KeyCode::Delete => tui_textarea::Key::Delete,
        KeyCode::F(f) => tui_textarea::Key::F(f),
        KeyCode::Char(c) => tui_textarea::Key::Char(c),
        KeyCode::Esc => tui_textarea::Key::Esc,
        _ => return None,
    };
    Some((key_code, key_event.modifiers))
}

struct RenderInput {
    text_area: RwSignal<TextArea<'static>>,
}

impl RenderNode for RenderInput {
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        self.text_area.with(|t| t.render(area, frame.buffer_mut()));
    }
}

impl WidgetRole for RenderInput {
    fn widget_role() -> Option<Role> {
        None
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
