mod client;
mod server;

use std::error::Error;
use std::time::Duration;

use client::{add_todo, delete_todo, fetch_todos, update_todo};
use rooibos::components::{Button, Input, InputRef, Notification, Notifications, Notifier, Show};
use rooibos::dom::{NodeId, WidgetState, focus_id};
use rooibos::reactive::graph::actions::Action;
use rooibos::reactive::graph::computed::AsyncDerived;
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::owner::{StoredValue, provide_context, use_context};
use rooibos::reactive::graph::signal::{ArcRwSignal, RwSignal};
use rooibos::reactive::graph::traits::{Get, Set, Track, With};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::layout::{
    align_items, block, chars, clear, justify_content, position, show,
};
use rooibos::reactive::{
    Errors, IntoAny, Render, RenderAny, UpdateLayoutProps, after_render, col, derive_signal,
    height, line, margin, margin_left, margin_top, max_width, mount, row, span, text, transition,
    wgt, width,
};
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::{Block, Paragraph};
use server::run_server;
use taffy::{AlignItems, JustifyContent, Position};

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353")
        .await
        .unwrap();

    mount(|| app(Duration::from_secs(3)));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    tokio::spawn(run_server(listener));
    runtime.run().await?;

    Ok(())
}

#[derive(Clone)]
struct TodoContext {
    add_todo: Action<String, Result<(), rooibos::dom::Error>>,
    update_todo: Action<(u32, String), Result<(), rooibos::dom::Error>>,
    delete_todo: Action<u32, Result<(), rooibos::dom::Error>>,
}

fn app(notification_timeout: Duration) -> impl Render {
    let editing_id = RwSignal::new(None);

    let add_todo = Action::new(move |text: &String| add_todo(text.clone()));
    let update_todo = Action::new(move |(id, text): &(u32, String)| update_todo(*id, text.clone()));
    let delete_todo = Action::new(move |id: &u32| delete_todo(*id));
    provide_context(TodoContext {
        add_todo,
        update_todo,
        delete_todo,
    });

    col![
        row![
            props(max_width!(100.), height!(3.)),
            wgt!(
                props(width!(12.), margin_top!(1.), margin_left!(1.)),
                "Add a Todo"
            ),
            create_todos_input()
        ],
        row![
            props(height!(100.%), block(Block::bordered().title("Todos"))),
            col![todos_body(editing_id, notification_timeout)]
        ],
        saving_popup(),
        Notifications::new().render()
    ]
}

fn create_todos_input() -> impl Render {
    let TodoContext { add_todo, .. } = use_context::<TodoContext>().unwrap();
    let input_ref = Input::get_ref();

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
            add_todo.dispatch(val);
            input_ref.delete_line_by_head();
        })
        .height(chars(3.))
        .render(input_ref)
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
        let notifier = notifier.clone();
        move || {
            if let Some(update_value) = update_todo.value().get() {
                let notification = match update_value {
                    Ok(()) => Notification::new(text!("", "  Todo updated", "")),
                    Err(e) => {
                        Notification::new(text!("", "  Failed to update todo", e.to_string()))
                    }
                };
                notifier.notify(notification.timeout(notification_timeout));
            }
        }
    });

    Effect::new(move || {
        if let Some(update_value) = delete_todo.value().get() {
            let notification = match update_value {
                Ok(()) => Notification::new(text!("", "  Todo deleted", "")),
                Err(e) => Notification::new(text!("", "  Failed to delete todo", e.to_string())),
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
        {
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
            })
        },
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

    let add_edit_id = StoredValue::new(NodeId::new_auto());

    let input_ref = Input::get_ref();

    row![
        props(height!(3.)),
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
    add_edit_id: StoredValue<NodeId>,
    editing_id: RwSignal<Option<u32>>,
    input_ref: InputRef,
) -> impl Render {
    let edit_save_text = derive_signal!(text!(if editing.get() {
        "".green()
    } else {
        "".blue()
    }));

    Button::new()
        .width(chars(5.))
        .id(add_edit_id.get_value())
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
    add_edit_id: StoredValue<NodeId>,
    input_ref: InputRef,
) -> impl Render {
    let TodoContext { update_todo, .. } = use_context::<TodoContext>().unwrap();

    let input_id = StoredValue::new(NodeId::new_auto());

    // Focus after the next render
    after_render(move || {
        focus_id(input_id.get_value());
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
            if blur_event.new_target != Some(add_edit_id.get_value()) {
                editing_id.set(None);
            }
        })
        .id(input_id.get_value())
        .render(input_ref)
}

#[cfg(test)]
mod tests;