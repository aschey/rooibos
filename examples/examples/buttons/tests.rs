use rooibos::dom::{root, KeyCode};
use rooibos::runtime::RuntimeSettings;
use rooibos::tester::TestHarness;

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
async fn test_buttons() {
    let mut harness = TestHarness::new(RuntimeSettings::default(), 20, 10, app);
    let root_node = root();
    assert_snapshot!(harness.terminal());
    let top_button = harness.find_all_by_text(&root_node, "count: 0")[0].clone();
    let button_rect = top_button.rect();
    harness.send_mouse_move(button_rect.x, button_rect.y);

    harness
        .wait_for(|_, harness| harness.find_by_text(&root_node, "╔").is_some())
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    harness.send_key(KeyCode::Tab);
    harness.send_key(KeyCode::Enter);

    harness
        .wait_for(|_, harness| harness.find_by_text(&root_node, "count: 1").is_some())
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    harness.exit().await;
}
