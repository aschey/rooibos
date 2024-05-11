use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{focus_next, render_dom, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{
    setup_terminal, start, tick, RuntimeSettings, TerminalSettings, TickResult,
};
use rooibos::tui::widgets::Paragraph;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    start(RuntimeSettings::default(), app);
    let mut terminal = setup_terminal::<Stdout>(TerminalSettings::default())?;

    terminal.draw(render_dom)?;
    loop {
        let tick_result = tick().await;
        match tick_result {
            TickResult::Redraw => {
                terminal.draw(render_dom)?;
            }
            TickResult::Exit => {
                return Ok(());
            }
            TickResult::Continue => {}
        }
    }
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    Effect::new(move |_| {
        focus_next();
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    widget_ref!(Paragraph::new(format!("count {}", count.get())))
        .focusable(true)
        .on_key_down(key_down)
}
