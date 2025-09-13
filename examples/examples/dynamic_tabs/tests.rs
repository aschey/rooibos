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
async fn test_tabs() {
    let mut harness = TestHarness::new(40, 10).await;
    harness.mount(app).await;

    assert_snapshot!(harness);

    let tab2_pos = harness.get_position_of_text("Tab2").await;
    harness.click_pos(tab2_pos);
    harness
        .wait_for(async |harness, _| harness.terminal_view().await.contains("tab2"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    let add_pos = harness.get_position_of_text("+").await;
    harness.click_pos(add_pos);
    harness
        .wait_for(async |harness, _| harness.terminal_view().await.contains("Tab3"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    let tab3_pos = harness.get_position_of_text("Tab3").await;
    harness.click_pos(tab3_pos);
    harness
        .wait_for(async |harness, _| harness.terminal_view().await.contains("tab3"))
        .await
        .unwrap();

    let tab2_close = harness.get_nth_position_of_text("✕", 2).await;
    harness.click_pos(tab2_close);
    harness
        .wait_for(async |harness, _| harness.terminal_view().await.contains("tab2"))
        .await
        .unwrap();
    assert_snapshot!(harness);

    harness.exit().await;
}
