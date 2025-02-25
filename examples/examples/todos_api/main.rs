use rooibos::keybind::{
    Bind, CommandFilter, KeybindContext, extract, key, keys, on_command, use_command_context,
};
mod client;
mod server;

use std::process::ExitCode;
use std::time::Duration;

use client::{add_todo, delete_todo, fetch_todos, update_todo};
use rooibos::components::{
    Button, Input, InputRef, Notification, Notifier, Show, use_notifications,
};
use rooibos::keybind::{CommandBar, CommandHandler, Commands};
use rooibos::reactive::any_view::IntoAny as _;
use rooibos::reactive::dom::layout::{
    Borders, absolute, align_items, borders, clear, full, height, justify_content, margin,
    overflow_y, padding, padding_left, position, scroll, show, width,
};
use rooibos::reactive::dom::{
    NodeId, Render, RenderAny, UpdateLayoutProps, after_render, focus_id, line, span, text,
    use_focus, use_focus_with_id,
};
use rooibos::reactive::graph::actions::Action;
use rooibos::reactive::graph::computed::AsyncDerived;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::owner::{provide_context, use_context};
use rooibos::reactive::graph::signal::{ArcRwSignal, RwSignal};
use rooibos::reactive::graph::traits::{Get, Set, Track, With};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{Errors, col, derive_signal, row, transition, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings, max_viewport_width};
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;
use server::run_server;
use taffy::{AlignItems, JustifyContent};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[derive(clap::Parser, Commands, Clone, Debug, PartialEq, Eq)]

enum Command {
    Add { val: String },
    Edit { id: u32 },
    Delete { id: u32 },
}

#[rooibos::main]
async fn main() -> Result {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353")
        .await
        .unwrap();
    tokio::spawn(run_server(listener));
    let mut cmd_handler = CommandHandler::<Command>::new();
    cmd_handler.generate_commands();

    Runtime::initialize_with(
        RuntimeSettings::default().handle_commands(cmd_handler),
        DefaultBackend::auto(),
    )
    .run(|| app(Duration::from_secs(3)))
    .await
}

#[derive(Clone)]
struct TodoContext {
    add_todo: Action<String, std::result::Result<(), rooibos::reactive::error::Error>>,
    update_todo: Action<(u32, String), std::result::Result<(), rooibos::reactive::error::Error>>,
    delete_todo: Action<u32, std::result::Result<(), rooibos::reactive::error::Error>>,
}

fn app(notification_timeout: Duration) -> impl Render {
    max_viewport_width(200).unwrap();

    let editing_id = RwSignal::new(None);

    let add_todo = Action::new(move |text: &String| add_todo(text.clone()));
    let update_todo = Action::new(move |(id, text): &(u32, String)| update_todo(*id, text.clone()));
    let delete_todo = Action::new(move |id: &u32| delete_todo(*id));

    on_command(extract!(val, Command::Add { val }), move |val| {
        add_todo.dispatch(val);
    });
    on_command(extract!(id, Command::Edit { id }), move |id| {
        editing_id.set(Some(id));
    });
    on_command(extract!(id, Command::Delete { id }), move |id| {
        delete_todo.dispatch(id);
    });

    provide_context(TodoContext {
        add_todo,
        update_todo,
        delete_todo,
    });

    let command_context = use_command_context();
    let input_id = NodeId::new_auto();

    let (notifications, notifier) = use_notifications();
    col![
        props(padding(1), width(full()), height(full())),
        row![
            props(width(full()), align_items(AlignItems::Center)),
            wgt!("Add a Todo"),
            add_todo_input(input_id)
        ],
        col![
            props(
                width(full()),
                height(full()),
                overflow_y(scroll()),
                borders(Borders::all().title("Todos"))
            ),
            todos_body(editing_id, notifier, notification_timeout)
        ],
        CommandBar::<Command>::new().render(),
        saving_popup(),
        notifications.render()
    ]
    .on_key_down(
        [
            key("a", move |_, _| {
                focus_id(input_id);
            }),
            // {dec+}e
            key(
                keys::combine([keys::Key::decimal('+'), keys::Key::Literal('e')]),
                move |_, context: KeybindContext| {
                    let id = context.keys[0].get_numeric();
                    command_context.dispatch(Command::Edit { id });
                },
            ),
            // {dec+}d
            key(
                keys::combine([keys::Key::decimal('+'), keys::Key::Literal('d')]),
                move |_, context: KeybindContext| {
                    let id = context.keys[0].get_numeric();
                    command_context.dispatch(Command::Delete { id });
                },
            ),
        ]
        .bind(),
    )
}

fn add_todo_input(id: NodeId) -> impl Render {
    let command_context = use_command_context::<Command>();
    let input_ref = Input::get_ref();
    let focused = use_focus_with_id(id);

    row![
        props(width(full()), padding_left(1)),
        Input::default()
            .borders(derive_signal!(if focused.get() {
                Borders::all().blue()
            } else {
                Borders::all()
            }))
            .placeholder_text("Add a todo")
            .grow(1.)
            .on_submit(move |val| {
                input_ref.delete_line();
                command_context.dispatch(Command::Add { val });
            })
            .min_width(12)
            .max_width(100)
            .id(id)
            .render(input_ref),
        Button::new()
            .on_click(move || {
                input_ref.submit();
            })
            .render(text!("submit"))
    ]
}

fn todos_body(
    editing_id: RwSignal<Option<u32>>,
    notifier: Notifier,
    notification_timeout: Duration,
) -> impl RenderAny {
    let TodoContext {
        update_todo,
        delete_todo,
        add_todo,
    } = use_context::<TodoContext>().unwrap();

    let add_version = add_todo.version();
    let update_version = update_todo.version();
    let delete_version = delete_todo.version();

    Effect::new({
        move || {
            if let Some(update_value) = update_todo.value().get() {
                let notification = match update_value {
                    Ok(()) => Notification::new("Todo updated"),
                    Err(e) => Notification::new(text!("Failed to update todo", e.to_string())),
                };
                notifier.notify(notification.timeout(notification_timeout));
            }
        }
    });

    Effect::new(move || {
        if let Some(update_value) = delete_todo.value().get() {
            let notification = match update_value {
                Ok(()) => Notification::new("Todo deleted"),
                Err(e) => Notification::new(text!("Failed to delete todo", e.to_string())),
            };
            notifier.notify(notification.timeout(notification_timeout));
        }
    });

    let todos = AsyncDerived::new(move || {
        add_version.track();
        update_version.track();
        delete_version.track();
        fetch_todos()
    });

    let fallback = move |errors: ArcRwSignal<Errors>| {
        let error_list =
            move || errors.with(|errors| errors.iter().map(|(_, e)| span!(e)).collect::<Vec<_>>());

        wgt!(line!(error_list()))
    };

    transition!(
        wgt!(line!(" Loading...".gray())),
        todos.await.map(|todos| {
            if todos.is_empty() {
                wgt!("No todos".gray()).into_any()
            } else {
                todos
                    .into_iter()
                    .map(|t| todo_item(t.id, t.text, editing_id))
                    .collect::<Vec<_>>()
                    .into_any()
            }
        }),
        fallback
    )
}

fn saving_popup() -> impl RenderAny {
    let TodoContext {
        update_todo,
        delete_todo,
        add_todo,
    } = use_context::<TodoContext>().unwrap();

    let add_pending = add_todo.pending();
    let update_pending = update_todo.pending();
    let delete_pending = delete_todo.pending();

    let pending = derive_signal!(add_pending.get() || update_pending.get() || delete_pending.get());

    row![
        props(
            width(full()),
            height(full()),
            position(absolute()),
            align_items(AlignItems::Center),
            justify_content(JustifyContent::Center),
            show(pending)
        ),
        wgt!(
            props(clear(true), width(25), height(5), borders(Borders::all())),
            line!("Saving...")
        )
    ]
}

fn todo_item(id: u32, text: String, editing_id: RwSignal<Option<u32>>) -> impl Render {
    let editing = derive_signal!(editing_id.get() == Some(id));
    let text = RwSignal::new(text);

    let add_edit_id = NodeId::new_auto();

    let input_ref = Input::get_ref();

    row![
        col![props(margin(1)), format!("{id}.")],
        add_edit_button(id, editing, add_edit_id, editing_id, input_ref),
        delete_button(id),
        Show::new()
            .fallback(move || col![props(margin(1)), wgt!(text.get())])
            .render(editing, move || {
                todo_editor(id, text, editing_id, add_edit_id, input_ref)
            })
    ]
}

fn add_edit_button(
    id: u32,
    editing: Signal<bool>,
    add_edit_id: NodeId,
    editing_id: RwSignal<Option<u32>>,
    input_ref: InputRef,
) -> impl Render {
    let edit_save_text = derive_signal!(text!(if editing.get() {
        "󱣪".green()
    } else {
        "󱞁".blue()
    }));

    Button::new()
        .id(add_edit_id)
        .padding_x(1)
        .centered()
        .on_click(move || {
            if editing.get() {
                input_ref.submit();
            } else {
                editing_id.set(Some(id));
            }
        })
        .render(edit_save_text)
}

fn delete_button(id: u32) -> impl Render {
    let TodoContext { delete_todo, .. } = use_context::<TodoContext>().unwrap();

    Button::new()
        .padding_x(1)
        .centered()
        .on_click(move || {
            delete_todo.dispatch(id);
        })
        .render(text!("x".red()))
}

fn todo_editor(
    id: u32,
    text: RwSignal<String>,
    editing_id: RwSignal<Option<u32>>,
    add_edit_id: NodeId,
    input_ref: InputRef,
) -> impl Render {
    let TodoContext { update_todo, .. } = use_context::<TodoContext>().unwrap();

    let (input_id, focused) = use_focus();

    // We can't focus until after rendering since the widget ID won't exist in the tree until then
    after_render(move || {
        focus_id(input_id);
    });

    Input::default()
        .borders(derive_signal!(if focused.get() {
            Borders::all().blue()
        } else {
            Borders::all().empty()
        }))
        .initial_value(text.get())
        .on_submit(move |value| {
            update_todo.dispatch((id, value));
            editing_id.set(None);
        })
        .on_blur(move |blur_event, _| {
            if blur_event.new_target != Some(add_edit_id.into()) {
                editing_id.set(None);
            }
        })
        .width(full())
        .grow(1.)
        .max_width(100)
        .id(input_id)
        .render(input_ref)
}

#[cfg(test)]
mod tests;
