use rooibos::keybind::{
    Bind, CommandFilter, KeybindContext, extract, key, keys, on_command, use_command_context,
};
use rooibos::reactive::graph::IntoReactiveValue;
mod client;
mod server;

use std::process::ExitCode;
use std::time::Duration;

use client::{add_todo, delete_todo, fetch_todos, update_todo};
use color_eyre::eyre::Result;
use rooibos::components::{Button, Input, InputRef, Notification, Notifications, Notifier, Show};
use rooibos::keybind::{CommandBar, CommandHandler, Commands};
use rooibos::reactive::any_view::IntoAny as _;
use rooibos::reactive::dom::layout::{
    Borders, absolute, align_items, background, borders, center, clear, focus_mode, full, height,
    justify_content, margin, overflow_y, padding, padding_left, position, scroll, show,
    vertical_list, width,
};
use rooibos::reactive::dom::{
    NodeId, Render, RenderAny, UpdateLayoutProps, after_render, focus_id, line, span, text,
    use_focus_with_id,
};
use rooibos::reactive::graph::actions::Action;
use rooibos::reactive::graph::computed::AsyncDerived;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::owner::{on_cleanup, provide_context, use_context};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set, Track};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{
    IntoText, StateProp, col, fallback, focus_scope, row, transition, use_state_prop, wgt,
};
use rooibos::runtime::{Runtime, RuntimeSettings, max_viewport_width};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::{Color, Stylize};
use server::run_server;

#[derive(clap::Parser, Commands, Clone, Debug, PartialEq, Eq)]

enum Command {
    Add { val: String },
    Edit { id: u32 },
    Delete { id: u32 },
}

#[rooibos::main]
async fn main() -> Result<ExitCode> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353").await?;
    tokio::spawn(run_server(listener));
    let mut cmd_handler = CommandHandler::<Command>::new();
    cmd_handler.generate_commands();

    let res = Runtime::initialize_with(
        RuntimeSettings::default().handle_commands(cmd_handler),
        DefaultBackend::auto().await?,
    )
    .run(|_| app(Duration::from_secs(3)))
    .await?;
    Ok(res)
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

    col![
        style(padding(1), width(full()), height(full())),
        focus_scope!(
            row![
                style(width(full()), align_items(center())),
                wgt!("Add a Todo"),
                add_todo_input(input_id)
            ],
            col![
                style(
                    width(full()),
                    height(full()),
                    overflow_y(scroll()),
                    borders(Borders::all().title("Todos"))
                ),
                todos_body(editing_id, notification_timeout)
            ],
        ),
        CommandBar::<Command>::new().render(),
        saving_popup(),
        Notifications::new().render()
    ]
    .on_key_down(
        [
            key("a", move |_, _| {
                focus_id(input_id);
            }),
            // {dec+}e
            key(
                keys::combine([keys::Key::decimal('+'), 'e'.into()]),
                move |_, context: KeybindContext| {
                    let id = context.keys[0].get_numeric();
                    command_context.dispatch(Command::Edit { id });
                },
            ),
            // {dec+}d
            key(
                keys::combine([keys::Key::decimal('+'), 'd'.into()]),
                move |_, context: KeybindContext| {
                    let id = context.keys[0].get_numeric();
                    command_context.dispatch(Command::Delete { id });
                },
            ),
        ]
        .bind(),
    )
    .focusable(false)
}

fn add_todo_input(id: NodeId) -> impl Render {
    let command_context = use_command_context::<Command>();
    let input_ref = Input::get_ref();
    let focused = use_focus_with_id(id);

    row![
        style(width(full()), padding_left(1)),
        Input::default()
            .borders(move || if focused.get() {
                Borders::all().blue()
            } else {
                Borders::all()
            })
            .placeholder_text("Add a todo")
            .flex_grow(1.)
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

fn todos_body(editing_id: RwSignal<Option<u32>>, notification_timeout: Duration) -> impl RenderAny {
    let TodoContext {
        update_todo,
        delete_todo,
        add_todo,
    } = use_context::<TodoContext>().unwrap();

    let add_version = add_todo.version();
    let update_version = update_todo.version();
    let delete_version = delete_todo.version();

    let notifier = Notifier::new();
    Effect::new(move || {
        if let Some(update_value) = update_todo.value().get() {
            let notification = match update_value {
                Ok(()) => Notification::new("Todo updated"),
                Err(e) => Notification::new(text!("Failed to update todo", e.to_string())),
            };
            notifier.notify(notification.timeout(notification_timeout));
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

    transition!(
        wgt!(line!(" Loading...".gray())),
        todos.await.map(|todos| {
            if todos.is_empty() {
                wgt!("No todos".gray()).into_any()
            } else {
                focus_scope!(
                    style(focus_mode(vertical_list())),
                    todos
                        .into_iter()
                        .map(|t| todo_item(t.id, t.text, editing_id))
                        .collect::<Vec<_>>()
                )
                .into_any()
            }
        }),
        |err| fallback(err, |e| span!(e).red())
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

    let pending = move || add_pending.get() || update_pending.get() || delete_pending.get();

    row![
        style(
            width(full()),
            height(full()),
            position(absolute()),
            align_items(center()),
            justify_content(center()),
            show(pending)
        ),
        wgt!(
            style(clear(true), width(25), height(5), borders(Borders::all())),
            line!("Saving...")
        )
    ]
}

fn todo_item(id: u32, text: String, editing_id: RwSignal<Option<u32>>) -> impl Render {
    let editing = move || editing_id.get() == Some(id);
    let text = RwSignal::new(text);

    let (row_bg, set_row_state) = use_state_prop(
        StateProp::new(Color::default())
            .focused(|_| Color::Indexed(237))
            .direct_focus(false),
    );

    let add_edit_id = NodeId::new_auto();
    let input_ref = Input::get_ref();

    row![
        style(background(row_bg)),
        col![style(margin(1)), format!("{id}.")],
        add_edit_button(id, editing, add_edit_id, editing_id, input_ref),
        delete_button(id),
        Show::new()
            .fallback(move || col![style(margin(1)), wgt!(text.get())])
            .render(editing, move || {
                todo_editor(id, text, editing_id, add_edit_id, input_ref)
            })
    ]
    .on_state_change(set_row_state)
}

fn add_edit_button<M>(
    id: u32,
    editing: impl IntoReactiveValue<Signal<bool>, M>,
    add_edit_id: NodeId,
    editing_id: RwSignal<Option<u32>>,
    input_ref: InputRef,
) -> impl Render {
    let editing = editing.into_reactive_value();
    let edit_save_text = move || {
        if editing.get() {
            "󱣪".green().into_text()
        } else {
            "󱞁".blue().into_text()
        }
    };

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

    let input_id = NodeId::new_auto();
    // We can't focus until after rendering since the widget ID won't exist in the tree until then
    after_render(move || {
        focus_id(input_id);
    });

    on_cleanup(move || {
        editing_id.set(None);
    });

    let (input_borders, set_input_state) = use_state_prop(
        StateProp::new(Borders::all().empty()).focused(|b: Borders| b.solid().blue()),
    );

    Input::default()
        .borders(input_borders)
        .initial_value(text.get())
        .on_submit(move |value| {
            update_todo.dispatch((id, value));
            editing_id.set(None);
        })
        .on_direct_blur(move |blur_event, _, _| {
            if blur_event.new_target != Some(add_edit_id.into()) {
                editing_id.set(None);
            }
        })
        .on_state_change(set_input_state)
        .width(full())
        .flex_grow(1.)
        .max_width(100)
        .id(input_id)
        .render(input_ref)
}

#[cfg(test)]
mod tests;
