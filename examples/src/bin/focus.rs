use std::error::Error;

use crossterm::event::KeyCode;
use rooibos::prelude::*;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::traits::Get;
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
    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                focus_prev();
            }
            if term_signal.code == KeyCode::Down {
                focus_next();
            }
        }
    });

    view! {
        <Row>
            <Col v:percentage=50>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 1"/>
                </Row>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 2"/>
                </Row>
            </Col>
            <Col v:percentage=50>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 3"/>
                </Row>
                <Row v:percentage=50>
                    <FocusBlock v:focusable=true title="item 4"/>
                </Row>
            </Col>
        </Row>
    }
}

#[component]
fn FocusBlock(#[prop(into)] title: &'static str) -> impl Render {
    let (id, focused) = use_focus();

    view! {
        <Paragraph v:id=id block=prop!(<Block/>)>
            {format!("{title} - focused: {}", focused.get())}
        </Paragraph>
    }
}
