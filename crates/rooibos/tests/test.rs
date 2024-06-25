use rooibos::dom::{widget_ref, Event, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::RuntimeSettings;
use rooibos::tester::{TerminalView, TestHarness};

macro_rules! assert_snapshot {
    ($terminal:expr) => {
        insta::with_settings!({
            snapshot_path => "../../../test/snapshots"
        }, {
            insta::assert_snapshot!($terminal.backend().buffer().terminal_view());
        });
    };
}

#[rooibos::test]
async fn test_counter_simple() {
    let mut harness = TestHarness::new(RuntimeSettings::default(), 20, 10, app);

    assert_snapshot!(harness.terminal());

    harness.send_event(Event::Key(KeyCode::Enter.into()));
    harness
        .wait_for(|buf| buf.terminal_view().contains("count 1"))
        .await
        .unwrap();
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    widget_ref!(format!("count {}", count.get()))
        .on_key_down(key_down)
        .on_click(move |_, _| update_count())
}
