use std::error::Error;

use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::run;

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

    Effect::new(move |_| {
        focus_next();
    });

    let key_down = move |key_event: KeyEvent| {
        if key_event.code == KeyCode::Enter {
            show_popup.update(|p| *p = !*p);
        }
    };

    view! {
        <overlay v:length=6>
            <paragraph
                v:focusable
                block=prop!(<Block borders=Borders::ALL/>)
                on:key_down=key_down
            >
                <Line>"text1"</Line>
                <Line>"text2"</Line>
                <Line>"text3"</Line>
                <Line>"text4"</Line>
            </paragraph>
            <Show
                when=move || show_popup.get()
            >
                {move || view! {
                    <Popup percent_x=50 percent_y=50> {
                        view! {
                            <paragraph v:length=3 block=prop!(<Block borders=Borders::ALL/>)>
                                "popup text"
                            </paragraph>
                            }
                        }
                    </Popup>
                }}
            </Show>
        </overlay>
    }
}
