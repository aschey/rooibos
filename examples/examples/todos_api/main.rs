use rooibos::keybind::{
    Bind, CommandFilter, KeybindContext, extract, handle_command, map_handler, use_command_context,
};
mod client;
mod server;

use std::process::ExitCode;
use std::time::Duration;

use client::{add_todo, delete_todo, fetch_todos, update_todo};
use rooibos::components::{
    Button, Input, InputRef, Notification, Notifications, Notifier, Show, provide_notifications,
};
use rooibos::keybind::{CommandBar, CommandHandler, Commands};
use rooibos::reactive::any_view::IntoAny as _;
use rooibos::reactive::dom::layout::{
    align_items, block, chars, clear, grow, justify_content, max_width, position, show,
};
use rooibos::reactive::dom::{
    NodeId, Render, RenderAny, UpdateLayoutProps, WidgetState, after_render, focus_id, line, mount,
    span, text,
};
use rooibos::reactive::graph::actions::Action;
use rooibos::reactive::graph::computed::AsyncDerived;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::owner::{provide_context, use_context};
use rooibos::reactive::graph::signal::{ArcRwSignal, RwSignal};
use rooibos::reactive::graph::traits::{Get, Set, Track, With};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{
    Errors, col, derive_signal, height, margin, margin_left, margin_top, padding_left, row,
    transition, wgt, width,
};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::{Block, Paragraph};
use server::run_server;
use taffy::{AlignItems, JustifyContent, Position};

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

    let mut cmd_handler = CommandHandler::<Command>::new();
    cmd_handler.generate_commands();

    mount(|| app(Duration::from_secs(3)));

    let runtime = Runtime::initialize_with(
        RuntimeSettings::default().handle_commands(cmd_handler),
        CrosstermBackend::stdout(),
    );

    tokio::spawn(run_server(listener));
    runtime.run().await
}

#[derive(Clone)]
struct TodoContext {
    add_todo: Action<String, std::result::Result<(), rooibos::reactive::Error>>,
    update_todo: Action<(u32, String), std::result::Result<(), rooibos::reactive::Error>>,
    delete_todo: Action<u32, std::result::Result<(), rooibos::reactive::Error>>,
}

fn app(notification_timeout: Duration) -> impl Render {
    provide_notifications();

    let editing_id = RwSignal::new(None);

    let add_todo = Action::new(move |text: &String| add_todo(text.clone()));
    let update_todo = Action::new(move |(id, text): &(u32, String)| update_todo(*id, text.clone()));
    let delete_todo = Action::new(move |id: &u32| delete_todo(*id));

    handle_command(extract!(val, Command::Add { val }), move |val| {
        add_todo.dispatch(val);
    });
    handle_command(extract!(id, Command::Edit { id }), move |id| {
        editing_id.set(Some(id));
    });
    handle_command(extract!(id, Command::Delete { id }), move |id| {
        delete_todo.dispatch(id);
    });

    provide_context(TodoContext {
        add_todo,
        update_todo,
        delete_todo,
    });

    let command_context = use_command_context();
    let input_id = NodeId::new_auto();
    let todos_max_width = chars(200.);

    col![
        props(max_width(todos_max_width)),
        row![
            props(height!(3.)),
            wgt!(
                props(width!(12.), margin_top!(1.), margin_left!(1.)),
                "Add a Todo"
            ),
            add_todo_input(input_id)
        ],
        row![
            props(height!(100.%), block(Block::bordered().title("Todos"))),
            col![todos_body(editing_id, notification_timeout)]
        ],
        CommandBar::<Command>::new().height(chars(1.)).render(),
        saving_popup(),
        Notifications::new()
            .max_layout_width(todos_max_width)
            .render()
    ]
    .on_key_down(
        [
            map_handler("a", move |_, _| {
                focus_id(input_id);
            }),
            map_handler("{dec+}e", move |_, context: KeybindContext| {
                let id = context.keys[0].get_numeric();
                command_context.dispatch(Command::Edit { id });
            }),
            map_handler("{dec+}d", move |_, context: KeybindContext| {
                let id = context.keys[0].get_numeric();
                command_context.dispatch(Command::Delete { id });
            }),
        ]
        .bind(),
    )
}

fn add_todo_input(id: NodeId) -> impl Render {
    let command_context = use_command_context::<Command>();
    let input_ref = Input::get_ref();

    row![
        props(grow(1.), padding_left!(1.)),
        Input::default()
            .placeholder_text("Add a todo")
            .grow(1.)
            .block(|state| {
                Block::bordered()
                    .fg(if state == WidgetState::Focused {
                        Color::Blue
                    } else {
                        Color::default()
                    })
                    .title("Input")
                    .into()
            })
            .on_submit(move |val| {
                input_ref.delete_line();
                command_context.dispatch(Command::Add { val });
            })
            .height(chars(3.))
            .min_width(chars(12.))
            .max_width(chars(100.))
            .id(id)
            .render(input_ref),
        Button::new()
            .width(chars(10.))
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

        wgt!(Paragraph::new(line!(error_list())))
    };

    transition!(
        wgt!(line!(" Loading...".gray())),
        todos.await.map(|todos| {
            col![if todos.is_empty() {
                wgt!("No todos".gray()).into_any()
            } else {
                todos
                    .into_iter()
                    .map(|t| todo_item(t.id, t.text, editing_id))
                    .collect::<Vec<_>>()
                    .into_any()
            }]
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
            width!(100.%),
            height!(100.%),
            position(Position::Absolute),
            align_items(AlignItems::Center),
            justify_content(JustifyContent::Center),
            show(pending)
        ),
        wgt!(
            props(clear(true), width!(25.), height!(5.)),
            Paragraph::new("Saving...").block(Block::bordered())
        )
    ]
}

fn todo_item(id: u32, text: String, editing_id: RwSignal<Option<u32>>) -> impl Render {
    let editing = derive_signal!(editing_id.get() == Some(id));
    let text = RwSignal::new(text);

    let add_edit_id = NodeId::new_auto();

    let input_ref = Input::get_ref();

    row![
        props(height!(3.)),
        col![props(margin!(1.), width!(3.)), format!("{id}.")],
        add_edit_button(id, editing, add_edit_id, editing_id, input_ref),
        delete_button(id),
        Show::new()
            .fallback(move || col![props(margin!(1.)), wgt!(Paragraph::new(text.get()))])
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
        .width(chars(5.))
        .id(add_edit_id)
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
        .width(chars(5.))
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

    Input::default()
        .block(|state| {
            Block::bordered()
                .fg(Color::Blue)
                .border_set(if state == WidgetState::Focused {
                    border::PLAIN
                } else {
                    border::EMPTY
                })
                .into()
        })
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
        .grow(1.)
        .max_width(chars(100.))
        .id(input_id)
        .render(input_ref)
}

#[cfg(test)]
mod tests;
