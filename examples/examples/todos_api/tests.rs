use std::time::Duration;

use rooibos::dom::{DomNodeRepr, KeyCode, Role, root};
use rooibos::reactive::mount;
use rooibos::runtime::RuntimeSettings;
use rooibos::tester::{TerminalView, TestHarness};

use crate::app;
use crate::server::run_server;

macro_rules! assert_snapshot {
    ($name:literal, $terminal:expr) => {
        insta::with_settings!({
            snapshot_path => "./snapshots"
        }, {
            insta::assert_debug_snapshot!($name, $terminal.backend().buffer());
        });
    };
}

#[rooibos::test]
async fn test_todos() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353")
        .await
        .unwrap();

    tokio::spawn(run_server(listener));
    mount(|| app(Duration::from_millis(500)));
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().enable_signal_handler(false),
        40,
        10,
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
    assert_snapshot!("init", harness.terminal());

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
    assert_snapshot!("after_create", harness.terminal());

    update_todo(&mut harness, &todos_block, "1", "todo 11", false).await;
    assert_snapshot!("keyboard_update", harness.terminal());
    wait_for_notification(&mut harness, "Todo updated").await;

    update_todo(&mut harness, &todos_block, "2", "todo 112", true).await;
    assert_snapshot!("click_update", harness.terminal());
    wait_for_notification(&mut harness, "Todo updated").await;

    // Delete todo
    harness.get_by_text(&todos_block, "x").click();
    harness
        .wait_for(|harness, _| harness.find_by_text(&todos_block, "No todos").is_some())
        .await
        .unwrap();
    wait_for_notification(&mut harness, "Todo deleted").await;
    assert_snapshot!("delete", harness.terminal());

    harness.exit().await;
}

async fn update_todo(
    harness: &mut TestHarness,
    todos_block: &DomNodeRepr,
    send_text: &str,
    new_text: &str,
    use_keyboard: bool,
) {
    // Update todo
    let edit_node = harness.get_by_text(todos_block, "");
    edit_node.click();
    harness
        .wait_for(|_, _| todos_block.find_all_by_role(Role::TextInput).len() == 1)
        .await
        .unwrap();
    harness.send_text(send_text);
    harness
        .wait_for(|harness, _| harness.find_by_text(todos_block, new_text).is_some())
        .await
        .unwrap();

    if use_keyboard {
        harness.send_key(KeyCode::Enter);
    } else {
        edit_node.click();
    }

    harness
        .wait_for(|harness, _| {
            todos_block.find_all_by_role(Role::TextInput).is_empty()
                && harness.find_by_text(todos_block, new_text).is_some()
        })
        .await
        .unwrap();
}

async fn wait_for_notification(harness: &mut TestHarness, text: &str) {
    harness
        .wait_for(|harness, _| !harness.buffer().terminal_view().contains(text))
        .await
        .unwrap();
}
