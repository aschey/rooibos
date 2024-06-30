use rooibos::dom::{root, Role};
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
async fn test_counter_simple() {
    let mut harness = TestHarness::new(RuntimeSettings::default(), 30, 10, app);
    let root_layout = root().find_by_id("root");
    let button = root_layout
        .find_by_role(Role::Button)
        .first()
        .unwrap()
        .clone();
    button.click();

    harness
        .wait_for(|buf| buf.terminal_view().contains("count: 0"))
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());
}
