use rooibos::dom::{send_event, Event, KeyCode};
use rooibos::runtime::RuntimeSettings;
use rooibos::tester::{TerminalView, TestHarness};

use crate::app;

macro_rules! assert_snapshot {
    ($terminal:expr) => {
        insta::with_settings!({
            snapshot_path => "./snapshots"
        }, {
            insta::assert_debug_snapshot!($terminal.backend().buffer());
        });
    };
}

#[rooibos::test]
async fn test_counter_simple() {
    let mut harness = TestHarness::new(RuntimeSettings::default(), 20, 10, app);

    assert_snapshot!(harness.terminal());

    send_event(Event::Key(KeyCode::Enter.into()));
    harness
        .wait_for(|buf| buf.terminal_view().contains("count: 1"))
        .await
        .unwrap();
}
