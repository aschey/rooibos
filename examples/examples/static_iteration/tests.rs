use rooibos::reactive::KeyCode;
use rooibos::runtime::RuntimeSettings;
use rooibos::tester::{TerminalView, TestHarness};

use crate::app;

macro_rules! assert_snapshot {
    ($harness:expr) => {
        insta::with_settings!({
            snapshot_path => "./snapshots"
        }, {
            insta::assert_debug_snapshot!($harness.buffer());
        });
    };
}

#[rooibos::test]
async fn test_counters() {
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().enable_signal_handler(false),
        20,
        20,
    )
    .await;
    harness.mount(app).await;

    assert_snapshot!(harness);

    harness.send_key(KeyCode::Char('+'));
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 1"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    harness.exit().await;
}
