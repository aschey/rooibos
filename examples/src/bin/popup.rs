use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{run, use_keypress};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(|| view!(<App/>));
    run().await?;
    Ok(())
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
