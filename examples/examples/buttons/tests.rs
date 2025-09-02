use rooibos::reactive::KeyCode;
use rooibos::reactive::dom::root;
use rooibos::tester::TestHarness;

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
async fn test_buttons() {
    let mut harness = TestHarness::new(25, 10).await;
    harness.mount(app).await;

    let root_node = root();
    assert_snapshot!(harness);
    let top_button = harness.find_by_text(&root_node, "bigger").unwrap();
    let button_rect = top_button.rect();
    harness.send_mouse_move(button_rect.x, button_rect.y);

    harness
        .wait_for(|harness, _| harness.find_by_text(&root_node, "╔").is_some())
        .await
        .unwrap();
    assert_snapshot!(harness);

    harness.send_key(KeyCode::Enter);

    harness
        .wait_for(|harness, _| harness.find_by_text(&root_node, "12 x 6").is_some())
        .await
        .unwrap();
    assert_snapshot!(harness);

    harness.exit().await;
}
