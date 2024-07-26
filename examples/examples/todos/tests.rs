use std::time::Duration;

use rooibos::dom::{root, KeyCode, Role};
use rooibos::runtime::RuntimeSettings;
use rooibos::tester::{TerminalView, TestHarness};

use crate::app;
use crate::server::run_server;

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
async fn test_todos() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353")
        .await
        .unwrap();

    tokio::spawn(run_server(listener));
    let mut harness = TestHarness::new(
        RuntimeSettings::default().enable_signal_handler(false),
        40,
        10,
        || app(Duration::from_millis(100)),
    );
    let root_layout = root();
    // Wait for initial data load
    harness
        .wait_for(|harness, _| {
            harness.find_by_text(&root_layout, "Input").is_some()
                && harness.find_by_text(&root_layout, "No todos").is_some()
        })
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    // Add todo
    let todo_input = root_layout.find_all_by_role(Role::TextInput)[0].clone();
    todo_input.focus();
    harness.send_text("todo 1");
    harness.send_key(KeyCode::Enter);

    let todos_block = harness.get_by_block_text(&root_layout, "Todos");
    harness
        .wait_for(|harness, _| harness.find_by_text(&todos_block, "todo 1").is_some())
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    // Update todo
    harness.get_by_text(&todos_block, "").click();
    harness
        .wait_for(|_, _| todos_block.find_all_by_role(Role::TextInput).len() == 1)
        .await
        .unwrap();
    harness.send_text("1");
    harness
        .wait_for(|harness, _| harness.find_by_text(&todos_block, "todo 11").is_some())
        .await
        .unwrap();
    harness.send_key(KeyCode::Enter);

    harness
        .wait_for(|harness, _| {
            todos_block.find_all_by_role(Role::TextInput).is_empty()
                && harness.find_by_text(&todos_block, "todo 11").is_some()
        })
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    // Wait for notification to complete
    harness
        .wait_for(|harness, _| !harness.buffer().terminal_view().contains("Todo updated"))
        .await
        .unwrap();

    // Delete todo
    harness.get_by_text(&todos_block, "x").click();
    harness
        .wait_for(|harness, _| harness.find_by_text(&todos_block, "No todos").is_some())
        .await
        .unwrap();
    assert_snapshot!(harness.terminal());

    harness.exit().await;
}
