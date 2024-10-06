use rooibos::dom::KeyCode;
use rooibos::reactive::mount;
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
    mount(|| {
        if cfg!(windows) {
            app("cmd".to_string(), vec!["/C".to_string(), "dir".to_string()])
        } else {
            app("ls".to_string(), Vec::new())
        }
    });
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().enable_signal_handler(false),
        40,
        10,
    );
    assert_snapshot!(harness.terminal());

    harness.send_key(KeyCode::Enter);

    // Wait for process to finish
    harness
        .wait_for(|_, last_tick_result| matches!(last_tick_result, Some(TickResult::Restart)))
        .await
        .unwrap();
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("Open Editor"))
        .await
        .unwrap();

    harness.exit().await;
}
