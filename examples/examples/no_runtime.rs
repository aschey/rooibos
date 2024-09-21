use std::error::Error;
use std::sync::Arc;

use rooibos::dom::{
    Event, KeyCode, KeyEvent, KeyModifiers, dispatch_event, dom_update_receiver, focus_next, line,
    render_dom, set_pixel_size, set_supports_keyboard_enhancement, span,
};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, mount, wgt};
use rooibos::terminal::Backend;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::backend::Backend as _;
use rooibos::tui::layout::Size;
use rooibos::tui::style::Stylize;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let backend = Arc::new(CrosstermBackend::stdout());
    let mut terminal = backend.setup_terminal()?;
    let window_size = terminal.backend_mut().window_size().ok();
    set_pixel_size(window_size.map(|s| Size {
        width: s.pixels.width / s.columns_rows.width,
        height: s.pixels.height / s.columns_rows.height,
    }))
    .unwrap();
    set_supports_keyboard_enhancement(backend.supports_keyboard_enhancement()).unwrap();

    mount(app);
    terminal.draw(|f| render_dom(f.buffer_mut()))?;
    let cancellation_token = CancellationToken::new();
    let (term_tx, mut term_rx) = broadcast::channel(32);
    focus_next();
    {
        let backend = backend.clone();
        let cancellation_token = cancellation_token.clone();
        tokio::spawn(async move {
            backend
                .read_input(term_tx, move || {
                    cancellation_token.clone().cancelled_owned()
                })
                .await;
        });
    }
    let mut dom_update_rx = dom_update_receiver();

    loop {
        tokio::select! {
            Ok(()) = dom_update_rx.changed() => {
                terminal.draw(|f| render_dom(f.buffer_mut()))?;
            }
            Ok(event) = term_rx.recv() => {
                    if should_exit(&event) {
                        break;
                    }

                    dispatch_event(event)
                }
            else => {
                break;
            }
        }
    }

    backend.restore_terminal()?;

    Ok(())
}

fn should_exit(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })
    )
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key_down)
        .on_click(move |_, _, _| update_count())
}