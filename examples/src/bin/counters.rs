use std::error::Error;

use crossterm::event::KeyCode;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{setup_terminal, tick, use_keypress, TickResult};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(|| view!(<Counters/>));

    loop {
        terminal
            .draw(|f: &mut Frame| {
                render_dom(f);
            })
            .unwrap();

        if tick().await == TickResult::Exit {
            return Ok(());
        }
    }
}

#[component]
fn Counter(id: u32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                set_count.update(|c| *c += 1);
            }
            if term_signal.code == KeyCode::Down {
                set_count.update(|c| *c -= 1);
            }
        }
    });

    view! {
        <Block
            v:id=id.to_string() title=format!("count: {}", count.get())
            v:constraint=constraint
        />
    }
}

#[component]
fn Counters() -> impl Render {
    let (n_counters, set_n_counters) = signal(1);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Enter {
                set_n_counters.update(|c| *c += 1);
            } else if term_signal.code == KeyCode::Backspace {
                set_n_counters.update(|c| *c -= 1);
            }
        }
    });

    view! {
        <Col>
            <ForEach
                each=move|| (0..n_counters.get())
                key=|i| *i
                children=move|i| {
                    view! {
                        <Counter id=i constraint=Constraint::Length(2)/>
                    }
                }
            />
        </Col>
    }
}
