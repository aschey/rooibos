use std::cell::RefCell;
use std::error::Error;
use std::fmt::format;
use std::io::{stdout, Stdout};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::Frame;
use rooibos::dom::prelude::*;
use rooibos::dom::{
    block, col, component, mount, print_dom, render_dom, row, view, BlockProps, Component,
    DocumentFragment, DomNode, IntoView, Mountable, ViewFragment,
};
use rooibos::reactive::{
    create_runtime, create_rw_signal, on_cleanup, RwSignal, SignalGet, SignalUpdate,
};
use rooibos::runtime::{create_key_effect, use_focus, Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = create_runtime();
    let mut rt = Runtime::initialize();
    let mut terminal = setup_terminal()?;
    mount(|| view!(<App/>));
    // print_dom(&mut std::io::stdout(), false);
    terminal.draw(|f: &mut Frame| {
        render_dom(f);
    })?;

    loop {
        if rt.tick().await == TickResult::Exit {
            restore_terminal(terminal)?;
            return Ok(());
        }
        terminal.draw(|f: &mut Frame| {
            render_dom(f);
        })?;
    }
}

fn setup_terminal() -> Result<Terminal> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

#[component]
fn App() -> impl IntoView {
    let show_popup = create_rw_signal(true);

    create_key_effect(move |event| {
        if event.code == KeyCode::Enter {
            show_popup.update(|p| *p = !*p);
        }
    });

    view! {
        <Overlay v:length=6>
            <Paragraph block=prop!(<Block borders=Borders::ALL/>)>
                <Line>"text1"</Line>
                <Line>"text2"</Line>
                <Line>"text3"</Line>
                <Line>"text4"</Line>
            </Paragraph>
            <Show
                when=move || show_popup.get()
            >
                {move || view! {
                    <Popup percent_x=50 percent_y=50>
                        {view! {
                            <Paragraph v:length=3 block=prop!(<Block borders=Borders::ALL/>)>
                                "popup text"
                            </Paragraph>
                            }
                        }
                    </Popup>
                }}
            </Show>
        </Overlay>
    }
}
