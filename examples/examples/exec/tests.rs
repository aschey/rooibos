use rooibos::reactive::KeyCode;
use rooibos::runtime::TickResult;
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
async fn test_exec() {
    let mut harness = TestHarness::new(40, 10).await;
    harness
        .mount((), |_| {
            if cfg!(windows) {
                app("cmd".to_string(), vec!["/C".to_string(), "dir".to_string()])
            } else {
                app("ls".to_string(), Vec::new())
            }
        })
        .await;
    assert_snapshot!(harness);

    harness.send_key(KeyCode::Enter);

    // Wait for process to finish
    harness
        .wait_for(async |_, last_tick_result| matches!(last_tick_result, Some(TickResult::Restart)))
        .await
        .unwrap();
    harness
        .wait_for(async |harness, _| harness.terminal_view().await.contains("Open Editor"))
        .await
        .unwrap();

    harness.exit().await;
}
