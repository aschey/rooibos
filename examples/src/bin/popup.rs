use std::error::Error;

use crossterm::event::KeyCode;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{setup_terminal, tick, use_keypress, TickResult};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal().unwrap();
    mount(|| view!(<App/>));

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
fn App() -> impl Render {
    let show_popup = RwSignal::new(false);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Enter {
                show_popup.update(|p| *p = !*p);
            }
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
