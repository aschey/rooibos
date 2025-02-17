use rooibos::reactive::tick;
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
async fn test_tabs() {
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().enable_signal_handler(false),
        40,
        10,
    )
    .await;
    harness.mount(app).await;

    assert_snapshot!(harness);

    let tab2_pos = harness.get_position_of_text("Tab2");
    harness.click_pos(tab2_pos);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("tab2"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    let add_pos = harness.get_position_of_text("+");
    harness.click_pos(add_pos);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("Tab3"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    let tab3_pos = harness.get_position_of_text("Tab3");
    harness.click_pos(tab3_pos);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("tab3"))
        .await
        .unwrap();

    let tab2_close = harness.get_nth_position_of_text("✕", 2);
    harness.click_pos(tab2_close);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("tab2"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    harness.exit().await;
}
