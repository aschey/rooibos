use std::error::Error;

use rooibos::prelude::Constraint::*;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::runtime::{run, use_keypress};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(counters);
    run().await?;
    Ok(())
}

fn counter(id: i32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);
    let default_padding = Padding {
        left: 1,
        top: 1,
        ..Default::default()
    };
    let block = RwSignal::new(Block::default().padding(default_padding));

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Up {
            set_count.update(|c| *c += 1);
        }
        if key_event.code == KeyCode::Down {
            set_count.update(|c| *c -= 1);
        }
    };

    widget_ref!(Paragraph::new(format!("count: {}", count.get())).block(block.get()))
        .constraint(constraint)
        .on_focus(move |_| block.set(Block::bordered().blue()))
        .on_blur(move |_| block.set(Block::default().padding(default_padding)))
        .on_key_down(key_down)
        .focusable(true)
        .id(id.to_string())
}

fn counters() -> impl Render {
    let (n_counters, set_n_counters) = signal(5);

    Effect::new(move |_| {
        focus_next();
    });

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

    col!(for_each(
        move || (0..n_counters.get()),
        |k| *k,
        |i| counter(i, Length(3))
    ))
}
