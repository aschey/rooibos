use rooibos::dom::{Role, root};
use rooibos::reactive::mount;
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
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().enable_signal_handler(false),
        30,
        10,
    );
    let root_layout = root().get_by_id("root");
    let add_button = root_layout
        .find_all_by_role(Role::Button)
        .first()
        .unwrap()
        .clone();
    add_button.click();

    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 0"))
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    harness.get_by_text(&root_layout, "+1").click();

    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 1"))
        .await
        .unwrap();

    add_button.click();

    harness
        .wait_for(|harness, _| harness.find_all_by_text(&root_layout, "+1").len() == 2)
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    harness.find_all_by_text(&root_layout, "x")[1].click();

    harness
        .wait_for(|harness, _| harness.find_all_by_text(&root_layout, "+1").len() == 1)
        .await
        .unwrap();

    harness.exit().await;
}
