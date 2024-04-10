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
    block, col, component, mount, print_dom, render_dom, row, view, BlockProps, DocumentFragment,
    DomNode,
};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{key_effect, use_focus, Runtime, TickResult};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    rooibos::runtime::execute(async_main).unwrap();
    Ok(())
}

#[tokio::main]
async fn async_main() -> Result<()> {
    rooibos::runtime::init(async move {
        let mut rt = Runtime::initialize();

        let mut terminal = setup_terminal().unwrap();
        mount(|| view!(<App/>), rt.connect_update());
        // print_dom(&mut std::io::stdout(), false);
        terminal
            .draw(|f: &mut Frame| {
                render_dom(f);
            })
            .unwrap();

        loop {
            if rt.tick().await == TickResult::Exit {
                restore_terminal(terminal).unwrap();
                return;
            }
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();
        }
    })
    .await;
    Ok(())
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
fn App() -> impl Render {
    let show_popup = RwSignal::new(false);

    key_effect(move |event| {
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
                    <Popup percent_x=50 percent_y=50> {
                        view! {
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
