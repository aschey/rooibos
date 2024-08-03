mod client;
mod server;

use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

use client::{add_todo, delete_todo, fetch_todos, update_todo};
use rooibos::components::{Button, Input, Notification, Notifications, Notifier, Popup, Show};
use rooibos::dom::{
    block, clear, col, derive_signal, fill, focus_id, length, line, margin, overlay, percentage,
    props, row, span, text, transition, widget_ref, Constrainable, Errors, IntoAny, NodeId, Render,
    WidgetState,
};
use rooibos::reactive::actions::Action;
use rooibos::reactive::computed::AsyncDerived;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::owner::{provide_context, use_context, StoredValue};
use rooibos::reactive::signal::{ArcRwSignal, RwSignal};
use rooibos::reactive::traits::{Get, Set, Track, With};
use rooibos::reactive::wrappers::read::Signal;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{wasm_compat, Runtime, RuntimeSettings};
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::{Block, Paragraph};
use server::run_server;

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9353")
        .await
        .unwrap();

    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        || app(Duration::from_secs(3)),
    );
    tokio::spawn(run_server(listener));
    runtime.run().await?;

    Ok(())
}

#[derive(Clone)]
struct TodoContext {
    update_todo: Action<(u32, String), Result<(), rooibos::dom::Error>>,
    delete_todo: Action<u32, Result<(), rooibos::dom::Error>>,
}

fn app(notification_timeout: Duration) -> impl Render {
    let fallback = move |errors: ArcRwSignal<Errors>| {
        let error_list =
            move || errors.with(|errors| errors.iter().map(|(_, e)| span!(e)).collect::<Vec<_>>());

        widget_ref!(Paragraph::new(line!(error_list())))
    };

    let editing_id = RwSignal::new(None);

    let add_todo = Action::new(move |text: &String| add_todo(text.clone()));
    let update_todo = Action::new(move |(id, text): &(u32, String)| update_todo(*id, text.clone()));
    let delete_todo = Action::new(move |id: &u32| delete_todo(*id));
    provide_context(TodoContext {
        update_todo,
        delete_todo,
    });

    let add_version = add_todo.version();
    let update_version = update_todo.version();
    let delete_version = delete_todo.version();

    let add_pending = add_todo.pending();
    let update_pending = update_todo.pending();
    let delete_pending = delete_todo.pending();

    let notifier = Notifier::new();

    Effect::new(move |_| {
        if update_version.get() > 0 {
            notifier.notify(
                Notification::new(text!("", "  Todo updated", "")).timeout(notification_timeout),
            );
        }
    });

    let todos = AsyncDerived::new(move || {
        // TODO: is this the best way to trigger a refetch?
        add_version.track();
        update_version.track();
        delete_version.track();
        fetch_todos()
    });
    let pending = derive_signal!(add_pending.get() || update_pending.get() || delete_pending.get());

    let input_ref = Input::get_ref();

    overlay![
        col![
            row![
                props!(length(3));
                col![
                    props!(length(12), block(Block::default()));
                    widget_ref!("Add a Todo")
                ],
                col![
                    props!(percentage(80));
                    Input::default()
                        .block(|state| Block::bordered()
                            .fg(if state == WidgetState::Focused {
                                Color::Blue
                            } else {
                                Color::default()
                            })
                            .title("Input")
                            .into())
                        .on_submit(move |val| {
                            add_todo.dispatch(val);
                            input_ref.delete_line_by_head();
                        })
                        .length(3)
                        .render(input_ref)
                ]
            ],
            row![
                props!(block(Block::bordered().title("Todos")));
                col![
                    transition!(
                        widget_ref!(line!(" Loading...".gray())),
                        {
                            todos.await.map(|todos| {
                                col![if todos.is_empty() {
                                    widget_ref!("No todos".gray()).into_any()
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
                ]
            ],
            Notifications::new().render()
        ],
        Popup::default()
            .percent_x(50)
            .percent_y(50)
            .render(pending, move || col![
                col![props!(fill(1));],
                clear![
                    props!(length(3));
                    widget_ref![
                        props!(block(Block::bordered()));
                        Paragraph::new("Saving...")
                    ]
                ],
                col![props!(fill(1));],
            ])
    ]
}

fn todo_item(id: u32, text: String, editing_id: RwSignal<Option<u32>>) -> impl Render {
    let editing = Signal::derive(move || editing_id.get() == Some(id));
    let text = RwSignal::new(text);
    let edit_save_text = derive_signal!(text!(if editing.get() {
        "".green()
    } else {
        "".blue()
    }));
    let add_edit_id = StoredValue::new(NodeId::new_auto());

    let TodoContext {
        update_todo,
        delete_todo,
    } = use_context::<TodoContext>().unwrap();

    let input_ref = Input::get_ref();
    let input_id = StoredValue::new(NodeId::new_auto());

    row![
        props!(length(3));
        Button::new()
            .length(5)
            .id(add_edit_id.get_value())
            .on_click(move || {
                if editing.get() {
                    input_ref.submit();
                } else {
                    editing_id.set(Some(id));
                }
            })
            .render(edit_save_text),
        Button::new()
            .length(5)
            .on_click(move || {
                delete_todo.dispatch(id);
            })
            .render(text!("x".red())),
        Show::new()
            .fallback(move || col![
                props!(margin(1));
                widget_ref!(Paragraph::new(text.get()))
            ])
            .render(editing, move || {
                // Focus after mounting
                wasm_compat::spawn_local(async move {
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
            })
    ]
}

#[cfg(test)]
mod tests;
