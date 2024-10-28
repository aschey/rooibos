use std::marker::PhantomData;

use modalkit::actions::{CommandBarAction, Editable, Promptable};
use modalkit::editing::application::ApplicationAction;
use modalkit_ratatui::TerminalCursor;
use modalkit_ratatui::cmdbar::CommandBarState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
use rooibos_dom::{focus_id, focus_prev};
use rooibos_reactive::dom::{DomWidget, LayoutProps, NodeId, Render, UpdateLayoutProps};
use rooibos_reactive::graph::signal::signal;
use rooibos_reactive::graph::traits::{Get, Set, Update, WriteValue};

use crate::{CommandCompleter, CommandHandlerAction, use_command_context};

pub struct CommandBar<T> {
    _phantom: PhantomData<T>,
    props: LayoutProps,
}

impl<T> UpdateLayoutProps for CommandBar<T> {
    fn update_props(mut self, props: LayoutProps) -> Self {
        self.props = props;
        self
    }

    fn layout_props(&self) -> LayoutProps {
        self.props.clone()
    }
}

impl<T> Default for CommandBar<T>
where
    T: ApplicationAction + CommandCompleter + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CommandBar<T>
where
    T: ApplicationAction + CommandCompleter + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            props: LayoutProps::default(),
            _phantom: Default::default(),
        }
    }

    pub fn render(self) -> impl Render {
        let id = NodeId::new_auto();
        let (focused, set_focused) = signal(false);
        let command_context = use_command_context::<T>();
        let store = command_context.store();
        let (state, set_state) = signal(CommandBarState::new(&mut store.write_value()));

        command_context.on_command_bar_action(move |action, context| match action {
            CommandHandlerAction::CommandBar(CommandBarAction::Focus(
                text,
                command_type,
                action,
            )) => {
                set_focused.set(true);
                set_state.update(|s| {
                    s.set_type(&text, command_type, &action, context);
                });
                focus_id(id);
                Vec::new()
            }
            CommandHandlerAction::CommandBar(CommandBarAction::Unfocus) => {
                set_focused.set(false);
                focus_prev();
                Vec::new()
            }
            CommandHandlerAction::Editor(editor_action) => {
                set_state.update(|s| {
                    s.editor_command(&editor_action, context, &mut store.write_value())
                        .unwrap();
                });
                vec![]
            }
            CommandHandlerAction::Prompt(prompt_action) => {
                let mut updates = Vec::new();
                set_state.update(|s| {
                    updates = s
                        .prompt(&prompt_action, context, &mut store.write_value())
                        .unwrap();
                });
                updates
            }
        });
        DomWidget::new::<CommandBar<T>, _, _>(move || {
            let mut state = state.get();
            let command_bar = modalkit_ratatui::cmdbar::CommandBar::new().focus(focused.get());

            move |rect: Rect, frame: &mut Frame| {
                command_bar
                    .clone()
                    .render(rect, frame.buffer_mut(), &mut state);
                if let Some((cx, cy)) = state.get_term_cursor() {
                    frame.set_cursor_position((cx, cy));
                }
            }
        })
        .layout_props(self.props)
        .id(id)
        .focusable(true)
    }
}
