use std::error::Error;
use std::sync::Arc;

use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::events::dispatch_event;
use rooibos::reactive::dom::{
    Render, dom_update_receiver, focus_next, line, mount, render_terminal, set_pixel_size,
    set_supports_keyboard_enhancement, span,
};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{self, CTRL, Event, KeyCode, NonblockingTerminal, Repeats, key, wgt};
use rooibos::terminal::{Backend, DefaultBackend};
use rooibos::theme::Stylize;
use tokio::sync::broadcast;
use tokio_stream::StreamExt as _;
use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let backend = Arc::new(DefaultBackend::auto().await?);
    let tui_backend = backend.create_tui_backend()?;
    let mut terminal = rooibos::tui::Terminal::new(tui_backend)?;
    backend.setup_terminal(terminal.backend_mut())?;

    let mut terminal = NonblockingTerminal::new(terminal);
    let window_size = terminal.window_size().await.ok();
    set_pixel_size(window_size).unwrap();
    set_supports_keyboard_enhancement(backend.supports_keyboard_enhancement()).unwrap();

    let window_size = terminal
        .with_terminal_mut({
            let backend = backend.clone();
            move |t| backend.window_size(t.backend_mut())
        })
        .await
        .ok();
    mount(app, window_size);
    render_terminal(&mut terminal).await?;
    let cancellation_token = CancellationToken::new();
    let (term_tx, mut term_rx) = broadcast::channel(32);
    focus_next();

    let cancellation_token = cancellation_token.clone();
    let mut input_stream = backend.async_input_stream(cancellation_token.clone());
    reactive::spawn(async move {
        while let Some(Some(event)) = input_stream
            .next()
            .with_cancellation_token(&cancellation_token)
            .await
        {
            let _ = term_tx.send(event);
        }
    });

    let mut dom_update_rx = dom_update_receiver();

    loop {
        tokio::select! {
            Ok(()) = dom_update_rx.changed() => {
                render_terminal(&mut terminal).await?;
            }
            Ok(event) = term_rx.recv() => {
                    if should_exit(&event) {
                        break;
                    }

                    dispatch_event(event.into())
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
    if let Some(key_event) = event.as_key_press(Repeats::Include) {
        matches!(key_event, key!(CTRL, KeyCode::Char('c')))
    } else {
        false
    }
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key(keys::ENTER, move |_, _| {
            update_count();
        }))
        .on_click(move |_| update_count())
}
