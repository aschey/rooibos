use std::error::Error;
use std::io::{stdout, Stdout};

use any_spawner::Executor;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use rooibos::dom::{
    dispatch_event, dom_update_receiver, focus_next, line, mount, render_dom, set_pixel_size,
    set_supports_keyboard_enhancement, span, unmount, wgt, Event, KeyCode, KeyEvent, KeyModifiers,
    Render,
};
use rooibos::reactive::owner::Owner;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::tui::backend::CrosstermBackend;
use rooibos::tui::layout::Size;
use rooibos::tui::prelude::Backend;
use rooibos::tui::style::Stylize;
use tokio::task;
use tokio_stream::StreamExt;

type Terminal = rooibos::tui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let owner = Owner::new();
    owner.with(async_main)
}

#[tokio::main]
async fn async_main() -> Result<()> {
    Executor::init_tokio()?;
    let local = task::LocalSet::new();
    local.run_until(run()).await
}

async fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;

    let window_size = terminal.backend_mut().window_size().ok();
    set_pixel_size(window_size.map(|s| Size {
        width: s.pixels.width / s.columns_rows.width,
        height: s.pixels.height / s.columns_rows.height,
    }))
    .unwrap();
    set_supports_keyboard_enhancement(
        crossterm::terminal::supports_keyboard_enhancement().unwrap(),
    )
    .unwrap();

    mount(app);
    terminal.draw(|f| render_dom(f.buffer_mut()))?;
    focus_next();

    let mut event_reader = EventStream::new().fuse();
    let mut dom_update_rx = dom_update_receiver();

    loop {
        tokio::select! {
            Ok(()) = dom_update_rx.changed() => {
                terminal.draw(|f| render_dom(f.buffer_mut()))?;
            }
            Some(Ok(event)) = event_reader.next() => {
                if let Ok(event) = event.try_into() {
                    if should_exit(&event) {
                        break;
                    }

                    dispatch_event(event)
                }
            }
            else => {
                break;
            }
        }
    }

    unmount();
    restore_terminal(terminal)?;

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

fn setup_terminal() -> Result<Terminal> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key_down)
        .on_click(move |_, _| update_count())
}
