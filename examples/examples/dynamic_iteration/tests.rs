use rooibos::reactive::KeyCode;
use rooibos::reactive::dom::root;
use rooibos::reactive::dom::widgets::Role;
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
        40,
        20,
    )
    .await;
    harness.mount(app).await;

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
    assert_snapshot!(harness);

    harness.send_key(KeyCode::Tab);

    let text_node = harness.get_by_text(&root_layout, "count: 0");

    harness
        .wait_for(|_, _| text_node.is_focused())
        .await
        .unwrap();

    harness.send_key(KeyCode::Char('+'));

    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 1"))
        .await
        .unwrap();

    assert_snapshot!(harness);

    add_button.click();

    harness
        .wait_for(|harness, _| harness.find_all_by_text(&root_layout, "count").len() == 2)
        .await
        .unwrap();
    assert_snapshot!(harness);

    harness.send_key(KeyCode::Tab);

    harness
        .wait_for(|_, _| text_node.is_focused())
        .await
        .unwrap();

    harness.send_key(KeyCode::Char('d'));

    harness
        .wait_for(|harness, _| harness.find_all_by_text(&root_layout, "count").len() == 1)
        .await
        .unwrap();

    assert_snapshot!(harness);

    harness.exit().await;
}
