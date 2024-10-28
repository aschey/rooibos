use rooibos::dom::KeyCode;
use rooibos::reactive::{mount, tick};
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
async fn test_counters() {
    mount(app);
    tick().await;
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().enable_signal_handler(false),
        20,
        20,
    );
    assert_snapshot!(harness.terminal());

    harness.send_key(KeyCode::Up);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 1"))
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    harness.exit().await;
}
