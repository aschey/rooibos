use rooibos::dom::KeyCode;
use rooibos::runtime::{RuntimeSettings, TickResult};
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
async fn test_exec() {
    let mut harness =
        TestHarness::new(RuntimeSettings::default(), 40, 10, || app("ls".to_string()));
    assert_snapshot!(harness.terminal());

    harness.send_key(KeyCode::Enter);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 1"))
        .await
        .unwrap();

    harness.send_key(KeyCode::Char('e'));
    harness
        .wait_for(|_, last_tick_result| matches!(last_tick_result, Some(TickResult::Restart)))
        .await
        .unwrap();

    harness.send_key(KeyCode::Enter);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 2"))
        .await
        .unwrap();

    harness.exit().await;
}
