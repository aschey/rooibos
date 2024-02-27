use std::borrow::BorrowMut;

use ratatui::backend::TestBackend;
use ratatui::prelude::{Buffer, Constraint, *};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use ratatui::Terminal;
use rooibos_dom::{
    block, col, component, list, mount, overlay, paragraph, prop, render_dom, row, stateful_list,
    tabs, view, BlockProps, BuildFacade, BuilderFacade, Component, ComponentChildren, DomNode,
    IntoView, ListProps, NewExt, ParagraphProps, StatefulListProps, TabsProps, View,
};
use rooibos_reactive::{create_runtime, store_value};
use typed_builder::TypedBuilder;

fn clear_style(buffer: &mut Buffer) {
    buffer.set_style(
        buffer.area,
        Style::default().remove_modifier(Modifier::all()),
    );
}

#[test]
fn standalone_widget() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let view = view! {
        <Block title="test" borders=Borders::ALL/>
    };

    let _ = create_runtime();

    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test────┐",
        "│        │",
        "└────────┘",
    ]));
}

#[test]
fn widget_no_props() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Block/>
        </Column>
    };

    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "          ",
        "          ",
        "          ",
    ]));
}

#[test]
fn simple_column() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Block title="test" borders=Borders::ALL/>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test────┐",
        "│        │",
        "└────────┘",
    ]));
}

#[test]
fn nested_layout() {
    let backend = TestBackend::new(10, 6);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Row v:length=4>
                <Block title="test1" borders=Borders::ALL/>
            </Row>
            <Row v:length=2>
                <Block title="test2" borders=Borders::ALL/>
            </Row>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test1───┐",
        "│        │",
        "│        │",
        "└────────┘",
        "┌test2───┐",
        "└────────┘",
    ]));
}

#[test]
fn conditional() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let a = 1;

    let view = view! {
        <Column>
            {
                match a {
                    1 => view!(<Block title="test" borders=Borders::ALL/>),
                    _ => view!(<Block title="test2" borders=Borders::ALL/>)
                }
            }
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test────┐",
        "│        │",
        "└────────┘",
    ]));
}

#[test]
fn list_basic() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <List>
                <ListItem>"test1"</ListItem>
                <ListItem>"test2"</ListItem>
            </List>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "test1     ",
        "test2     ",
        "          ",
    ]));
}

#[test]
fn prop_iteration() {
    let backend = TestBackend::new(10, 6);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <List>
                {
                    (0..5).map(|i| prop!(<ListItem>{format!("test{i}")}</ListItem>))
                        .collect::<Vec<_>>()
                }
            </List>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "test0     ",
        "test1     ",
        "test2     ",
        "test3     ",
        "test4     ",
        "          ",
    ]));
}

#[test]
fn stateful() {
    let _ = create_runtime();

    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = ListState::default();
    let view = view! {
        <StatefulList v:state=state.clone()>
            <ListItem>"test1"</ListItem>
            <ListItem>"test2"</ListItem>
        </StatefulList>
    };

    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "test1     ",
        "test2     ",
        "          ",
    ]));
}

#[test]
fn list_styled() {
    let backend = TestBackend::new(15, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <List>
                <ListItem style=prop!(<Style fg=Color::Black/>)>"test1"</ListItem>
                <ListItem>"test2"</ListItem>
            </List>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();

    let mut expected = Buffer::with_lines(vec![
        "test1          ",
        "test2          ",
        "               ",
    ]);

    for x in 0..15 {
        expected.get_mut(x, 0).set_fg(Color::Black);
    }
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&expected);
}

#[test]
fn block_children() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Tabs>
                "tab1"
                "tab2"
            </Tabs>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
}

#[test]
fn single_child_as_vec() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Tabs>
                <>{"tab1"}</>
            </Tabs>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
}

#[test]
fn single_nested_child_as_vec() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Tabs>
                <>
                    <Line>
                        <Span>"tab1"</Span>
                    </Line>
                </>
            </Tabs>
        </Column>
    };

    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
}

#[test]
fn complex_block_children() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Tabs select=0>
                <Line>"tab1"</Line>
                <Line>{vec![Span::from("tab2")]}</Line>
            </Tabs>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
}

#[test]
fn macro_as_prop() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Paragraph block=prop!{<Block borders=Borders::ALL/>}>
                "test"
            </Paragraph>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌────────┐",
        "│test    │",
        "└────────┘",
    ]));
}

#[test]
fn simple_overlay() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Overlay>
            <Block borders=Borders::ALL/>
            <Paragraph>
                "test"
            </Paragraph>
        </Overlay>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "test─────┐",
        "│        │",
        "└────────┘",
    ]));
}

#[test]
fn overlay_multiple() {
    let backend = TestBackend::new(10, 6);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Overlay>
            <Block borders=Borders::ALL title="test"/>
            <Column margin=1>
                <List v:length=2>
                    <ListItem>{"hi"}</ListItem>
                    <ListItem>{"yo"}</ListItem>
                </List>
                <List>
                    <ListItem>{"hi2"}</ListItem>
                    <ListItem>{"yo2"}</ListItem>
                </List>
            </Column>
        </Overlay>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test────┐",
        "│hi      │",
        "│yo      │",
        "│hi2     │",
        "│yo2     │",
        "└────────┘",
    ]));
}

#[test]
fn two_overlays() {
    let backend = TestBackend::new(10, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Column v:percentage=50>
                <Overlay>
                    <Block borders=Borders::ALL title="test"/>
                    <Column margin=1>
                        <List v:length=2>
                            <ListItem>{"hi"}</ListItem>
                            <ListItem>{"yo"}</ListItem>
                        </List>
                    </Column>
                </Overlay>
            </Column>
            <Column v:percentage=50>
                <Overlay>
                    <Block borders=Borders::ALL title="test2"/>
                    <Column margin=1>
                        <List v:length=2>
                            <ListItem>{"hi2"}</ListItem>
                            <ListItem>{"yo2"}</ListItem>
                        </List>
                    </Column>
                </Overlay>
            </Column>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test────┐",
        "│hi      │",
        "│yo      │",
        "└────────┘",
        "┌test2───┐",
        "│hi2     │",
        "│yo2     │",
        "└────────┘",
    ]));
}

#[test]
fn array_as_variable() {
    let _ = create_runtime();

    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let tab_items = store_value(vec!["tab1", "tab2"]);
    let view = view! {
        <Column>
            <Tabs>
                {tab_items.get_value()}
            </Tabs>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
}

#[test]
fn simple_custom_component() {
    #[component]
    fn Viewer(#[prop(into)] text: String, flag: bool) -> impl IntoView {
        let text = store_value(text);
        view! {
            <List>
                <>
                    <ListItem>{format!("{}={flag}", text.get_value())}</ListItem>
                </>
            </List>
        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(7, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Viewer text="hi" flag=true/>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi=true"]));
}

#[test]
fn custom_component_children() {
    #[component]
    fn Viewer(#[prop(into, children)] text: String) -> impl IntoView {
        let text = store_value(text);
        view! {
            <List>
                <>
                    <ListItem>{text.get_value()}</ListItem>
                </>
            </List>
        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Viewer>
                "hi"
            </Viewer>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi"]));
}

#[test]
fn generic_component() {
    #[component]
    fn Viewer<T: 'static>(#[prop(into)] text: String, flag: bool) -> impl IntoView {
        let _ = std::any::type_name::<T>();
        let text = store_value(text);
        view! {
            <List>
                <>
                    <ListItem>{format!("{}={flag}", text.get_value())}</ListItem>
                </>
            </List>
        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(7, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Viewer<usize> text="hi" flag=true/>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi=true"]));
}

#[test]
fn custom_component_children_second() {
    #[component]
    fn Viewer(
        #[prop(default = 0)] _something: usize,
        #[prop(into, children)] text: String,
        #[prop(default = 0)] _something_else: usize,
    ) -> impl IntoView {
        let text = store_value(text);
        view! {
            <List>
                <>
                    <ListItem>{text.get_value()}</ListItem>
                </>
            </List>
        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Viewer>
                "hi"
            </Viewer>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi"]));
}

#[test]
fn custom_child_prop() {
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp {
        #[children]
        #[builder(setter(into))]
        text: String,
    }

    #[component]
    fn Viewer(#[prop(into, children)] children: ChildProp) -> impl IntoView {
        let text = store_value(children.text);
        view! {
            <List>
                <>
                    <ListItem>{text.get_value()}</ListItem>
                </>
            </List>

        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Viewer>
                <ChildProp>{"hi"}</ChildProp>
            </Viewer>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi"]));
}

#[test]
fn component_child() {
    #[component]
    fn Viewer<V: IntoView>(#[prop(children)] children: V) -> impl IntoView {
        view! {
            <Column>
                {children}
            </Column>
        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Viewer>
                {view! {
                    <List>
                        <>
                            <ListItem>{"hi"}</ListItem>
                        </>
                    </List>
                }}
            </Viewer>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi"]));
}

#[test]
fn component_child_nested() {
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp<V: IntoView> {
        #[children]
        views: V,
    }

    #[component]
    fn Viewer<V: IntoView>(#[prop(children)] children: ChildProp<V>) -> impl IntoView {
        view! {
            <Column>
                {children.views}
            </Column>
        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(3, 2);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Viewer>
                <ChildProp> {
                    view! {
                        <List>
                            <ListItem>{"hi"}</ListItem>
                            <ListItem>{"bye"}</ListItem>
                        </List>
                    }
                }
                </ChildProp>
            </Viewer>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi", "bye"]));
}

#[test]
fn custom_component_nested_layout() {
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp<V: IntoView> {
        #[children]
        views: V,
    }

    #[component]
    fn Viewer<V: IntoView>(#[prop(children)] children: ChildProp<V>) -> impl IntoView {
        view! {
            <Column>
                {children.views}
            </Column>
        }
    }

    let _ = create_runtime();

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let view = view! {
        <Column>
            <Row v:length=1>
                <Viewer>
                    <ChildProp> {
                        view! {
                            <List>
                                <>
                                    <ListItem>{"hi"}</ListItem>
                                </>
                            </List>
                        }
                    }
                    </ChildProp>
                </Viewer>
            </Row>
        </Column>
    };
    mount(|| view);
    terminal
        .draw(|f: &mut Frame| {
            render_dom(f);
        })
        .unwrap();
    clear_style(terminal.backend_mut().buffer_mut());
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi"]));
}
