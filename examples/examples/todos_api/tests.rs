use std::time::Duration;

use rooibos::keybind::{CommandFilter, CommandHandler};
use rooibos::reactive::KeyCode;
use rooibos::reactive::dom::widgets::Role;
use rooibos::reactive::dom::{DomNodeRepr, root};
use rooibos::runtime::RuntimeSettings;
use rooibos::tester::TestHarness;

use crate::server::run_server;
use crate::{Command, app};

macro_rules! assert_snapshot {
    ($name:literal, $harness:expr) => {
        let buffer = $harness.buffer().await;
        insta::with_settings!({
            snapshot_path => "./snapshots"
        }, {
            insta::assert_debug_snapshot!($name, buffer);
        });
    };
}

#[rooibos::test]
async fn test_todos() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353")
        .await
        .unwrap();

    let mut cmd_handler = CommandHandler::<Command>::new();
    cmd_handler.generate_commands();

    tokio::spawn(run_server(listener));
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().handle_commands(cmd_handler),
        50,
        15,
    )
    .await;
    harness.mount(|| app(Duration::from_millis(500))).await;

    let root_layout = root();
    // Wait for initial data load
    harness
        .wait_for(async |harness, _| {
            harness
                .find_by_text(&root_layout, "Add a todo")
                .await
                .is_some()
                && harness
                    .find_by_text(&root_layout, "No todos")
                    .await
                    .is_some()
        })
        .await
        .unwrap();
    assert_snapshot!("init", harness);

    // Add todo
    let todo_input = root_layout.find_all_by_role(Role::TextInput)[0].clone();
    todo_input.focus();
    harness.send_text("todo 1");
    harness.send_key(KeyCode::Enter);

    let todos_block = harness.get_by_block_text(&root_layout, "Todos").await;
    harness
        .wait_for(async |harness, _| harness.find_by_text(&todos_block, "todo 1").await.is_some())
        .await
        .unwrap();
    assert_snapshot!("after_create", harness);

    update_todo(&mut harness, &todos_block, "1", "todo 11", false).await;
    assert_snapshot!("keyboard_update", harness);
    wait_for_notification(&mut harness, "Todo updated").await;

    update_todo(&mut harness, &todos_block, "2", "todo 112", true).await;
    assert_snapshot!("click_update", harness);
    wait_for_notification(&mut harness, "Todo updated").await;

    // Delete todo
    harness.get_by_text(&todos_block, "x").await.click();
    harness
        .wait_for(async |harness, _| {
            harness
                .find_by_text(&todos_block, "No todos")
                .await
                .is_some()
        })
        .await
        .unwrap();
    wait_for_notification(&mut harness, "Todo deleted").await;
    assert_snapshot!("delete", harness);

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
    let edit_node = harness.get_by_text(todos_block, "󱞁").await;
    edit_node.click();
    harness
        .wait_for(async |_, _| todos_block.find_all_by_role(Role::TextInput).len() == 1)
        .await
        .unwrap();
    harness.send_text(send_text);
    harness
        .wait_for(async |harness, _| harness.find_by_text(todos_block, new_text).await.is_some())
        .await
        .unwrap();

    if use_keyboard {
        harness.send_key(KeyCode::Enter);
    } else {
        edit_node.click();
    }

    harness
        .wait_for(async |harness, _| {
            todos_block.find_all_by_role(Role::TextInput).is_empty()
                && harness.find_by_text(todos_block, new_text).await.is_some()
        })
        .await
        .unwrap();
}

async fn wait_for_notification(harness: &mut TestHarness, text: &str) {
    harness
        .wait_for(async |harness, _| !harness.terminal_view().await.contains(text))
        .await
        .unwrap();
}
