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
    mount(|| view!(<Counters/>));
    run().await?;
    Ok(())
}

#[component]
fn Counter(id: i32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);
    let default_padding = Padding {
        left: 1,
        top: 1,
        ..Default::default()
    };
    let block = RwSignal::new(Block::default().padding(default_padding));

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Up {
            set_count.update(|c| *c += 1);
        }
        if key_event.code == KeyCode::Down {
            set_count.update(|c| *c -= 1);
        }
    };

    view! {
        <paragraph
            block=block.get()
            v:id=id.to_string()
            v:focusable
            v:constraint=constraint
            on:key_down=key_down
            on:focus=move || block.set(Block::bordered().blue())
            on:blur=move || block.set(Block::default().padding(default_padding))
        >
            {format!("count: {}", count.get())}
        </paragraph>
    }
}

#[component]
fn Counters() -> impl Render {
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

    view! {
        <col>
            <For
                each=move|| (0..n_counters.get())
                key=|i| *i
                children=move|i| {
                    view! {
                        <Counter id=i constraint=Length(3)/>
                    }
                }
            />
        </col>
    }
}
