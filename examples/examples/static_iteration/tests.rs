use rooibos::reactive::KeyCode;
use rooibos::tester::TestHarness;

use crate::app;

macro_rules! assert_snapshot {
    ($harness:expr) => {
        let buffer = $harness.buffer().await;
        insta::with_settings!({
            snapshot_path => "./snapshots"
        }, {
            insta::assert_debug_snapshot!(buffer);
        });
    };
}

#[rooibos::test]
async fn test_counters() {
    let mut harness = TestHarness::new(20, 20).await;
    harness.mount((), |_| app()).await;

    assert_snapshot!(harness);

    harness.send_key(KeyCode::Char('+'));
    harness
        .wait_for(async |harness, _| harness.terminal_view().await.contains("count: 1"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    harness.exit().await;
}
