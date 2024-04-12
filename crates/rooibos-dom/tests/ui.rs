use ctor::ctor;
use ratatui::backend::TestBackend;
use ratatui::prelude::{Buffer, Constraint, *};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use ratatui::Terminal;
use reactive_graph::owner::StoredValue;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Get;
use rooibos_dom::{
    block, col, component, list, mount, overlay, paragraph, prop, render_dom, row, stateful_list,
    tabs, view, BlockProps, BuildFacade, BuilderFacade, ComponentChildren, DomNode, ListProps,
    NewExt, ParagraphProps, Render, StatefulListProps, TabsProps, View,
};
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::LocalSet;
use typed_builder::TypedBuilder;

#[ctor]
fn foo() {
    any_spawner::Executor::init_tokio().unwrap();
}

#[tokio::test]
async fn standalone_widget() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 3);
            let mut terminal = Terminal::new(backend).unwrap();
            let view = view! {
                <Block title="test" borders=Borders::ALL/>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "┌test────┐",
                "│        │",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn widget_no_props() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 3);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Block/>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "          ",
                "          ",
                "          ",
            ]));
        })
        .await;
}

#[tokio::test]
async fn simple_column() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 3);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Block title="test" borders=Borders::ALL/>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "┌test────┐",
                "│        │",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn str_only() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(4, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    "test"
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["test"]));
        })
        .await;
}

#[tokio::test]
async fn str_block() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(4, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    {"test"}
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["test"]));
        })
        .await;
}

#[tokio::test]
async fn string_block() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(4, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    {"test".to_string()}
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["test"]));
        })
        .await;
}

#[tokio::test]
async fn nested_layout() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 6);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Row v:length=4>
                        <Block title="test1" borders=Borders::ALL/>
                    </Row>
                    <Row v:length=2>
                        <Block title="test2" borders=Borders::ALL/>
                    </Row>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "┌test1───┐",
                "│        │",
                "│        │",
                "└────────┘",
                "┌test2───┐",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn ratio() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 6);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Row v:ratio=(2,3)>
                        <Block title="test1" borders=Borders::ALL/>
                    </Row>
                    <Row v:min=0>
                        <Block title="test2" borders=Borders::ALL/>
                    </Row>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "┌test1───┐",
                "│        │",
                "│        │",
                "└────────┘",
                "┌test2───┐",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn conditional() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 3);
            let mut terminal = Terminal::new(backend).unwrap();
            let a = 1;

            let view = view! {
                <Col>
                    {
                        match a {
                            1 => view!(<Block title="test" borders=Borders::ALL/>),
                            _ => view!(<Block title="test2" borders=Borders::ALL/>)
                        }
                    }
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "┌test────┐",
                "│        │",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn list_basic() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 3);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <List>
                        <ListItem>"test1"</ListItem>
                        <ListItem>"test2"</ListItem>
                    </List>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "test1     ",
                "test2     ",
                "          ",
            ]));
        })
        .await;
}

#[tokio::test]
async fn prop_iteration() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 6);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <List>
                        {
                            (0..5).map(|i| prop!(<ListItem>{format!("test{i}")}</ListItem>))
                                .collect::<Vec<_>>()
                        }
                    </List>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "test0     ",
                "test1     ",
                "test2     ",
                "test3     ",
                "test4     ",
                "          ",
            ]));
        })
        .await;
}

#[tokio::test]
async fn stateful() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 3);
            let mut terminal = Terminal::new(backend).unwrap();

            let state = RwSignal::new(ListState::default());
            let view = view! {
                <StatefulList v:state=move || state.get()>
                    <ListItem>"test1"</ListItem>
                    <ListItem>"test2"</ListItem>
                </StatefulList>
            };

            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "test1     ",
                "test2     ",
                "          ",
            ]));
        })
        .await;
}

#[tokio::test]
async fn list_styled() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(15, 3);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <List>
                        <ListItem style=prop!(<Style fg=Color::Black/>)>"test1"</ListItem>
                        <ListItem>"test2"</ListItem>
                    </List>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
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

            terminal.backend().assert_buffer(&expected);
        })
        .await;
}

#[tokio::test]
async fn block_children() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(15, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Tabs highlight_style=Style::default()>
                        "tab1"
                        "tab2"
                    </Tabs>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
        })
        .await;
}

#[tokio::test]
async fn single_child_as_vec() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(15, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Tabs highlight_style=Style::default()>
                        <>{"tab1"}</>
                    </Tabs>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
        })
        .await;
}

#[tokio::test]
async fn single_nested_child_as_vec() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(15, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Tabs highlight_style=Style::default()>
                        <>
                            <Line>
                                <Span>"tab1"</Span>
                            </Line>
                        </>
                    </Tabs>
                </Col>
            };

            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
        })
        .await;
}

#[tokio::test]
async fn complex_block_children() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(15, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Tabs select=0 highlight_style=prop!(<Style/>)>
                        <Line>"tab1"</Line>
                        <Line>{vec![Span::from("tab2")]}</Line>
                    </Tabs>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
        })
        .await;
}

#[tokio::test]
async fn macro_as_prop() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 3);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Paragraph block=prop!{<Block borders=Borders::ALL/>}>
                        "test"
                    </Paragraph>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "┌────────┐",
                "│test    │",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn simple_overlay() {
    LocalSet::new()
        .run_until(async move {
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
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "test─────┐",
                "│        │",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn overlay_multiple() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 6);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Overlay>
                    <Block borders=Borders::ALL title="test"/>
                    <Col margin=|| 1>
                        <List>
                            <ListItem>{"hi"}</ListItem>
                            <ListItem>{"yo"}</ListItem>
                        </List>
                        <List>
                            <ListItem>{"hi2"}</ListItem>
                            <ListItem>{"yo2"}</ListItem>
                        </List>
                    </Col>
                </Overlay>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal.backend().assert_buffer(&Buffer::with_lines(vec![
                "┌test────┐",
                "│hi      │",
                "│yo      │",
                "│hi2     │",
                "│yo2     │",
                "└────────┘",
            ]));
        })
        .await;
}

#[tokio::test]
async fn two_overlays() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(10, 8);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Col v:percentage=50>
                        <Overlay>
                            <Block borders=Borders::ALL title="test"/>
                            <Col margin=||1>
                                <List v:length=2>
                                    <ListItem>{"hi"}</ListItem>
                                    <ListItem>{"yo"}</ListItem>
                                </List>
                            </Col>
                        </Overlay>
                    </Col>
                    <Col v:percentage=50>
                        <Overlay>
                            <Block borders=Borders::ALL title="test2"/>
                            <Col margin=||1>
                                <List v:length=2>
                                    <ListItem>{"hi2"}</ListItem>
                                    <ListItem>{"yo2"}</ListItem>
                                </List>
                            </Col>
                        </Overlay>
                    </Col>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

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
        })
        .await;
}

#[tokio::test]
async fn array_as_variable() {
    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(15, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let tab_items = StoredValue::new(vec!["tab1", "tab2"]);
            let view = view! {
                <Col>
                    <Tabs highlight_style=Style::default()>
                        {tab_items.get().unwrap()}
                    </Tabs>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
        })
        .await;
}

#[tokio::test]
async fn simple_custom_component() {
    #[component]
    fn Viewer(#[prop(into)] text: String, flag: bool) -> impl Render {
        let text = StoredValue::new(text);
        view! {
            <List>
                <>
                    <ListItem>{format!("{}={flag}", text.get().unwrap())}</ListItem>
                </>
            </List>
        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(7, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Viewer text="hi" flag=true/>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi=true"]));
        })
        .await;
}

#[tokio::test]
async fn custom_component_children() {
    #[component]
    fn Viewer(#[prop(into, children)] text: String) -> impl Render {
        let text = StoredValue::new(text);
        view! {
            <List>
                <>
                    <ListItem>{text.get().unwrap()}</ListItem>
                </>
            </List>
        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(2, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Viewer>
                        "hi"
                    </Viewer>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi"]));
        })
        .await;
}

#[tokio::test]
async fn generic_component() {
    #[component]
    fn Viewer<T: 'static>(#[prop(into)] text: String, flag: bool) -> impl Render {
        let _ = std::any::type_name::<T>();
        let text = StoredValue::new(text);
        view! {
            <List>
                <>
                    <ListItem>{format!("{}={flag}", text.get().unwrap())}</ListItem>
                </>
            </List>
        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(7, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Viewer<usize> text="hi" flag=true/>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi=true"]));
        })
        .await;
}

#[tokio::test]
async fn custom_component_children_second() {
    #[component]
    fn Viewer(
        #[prop(default = 0)] _something: usize,
        #[prop(into, children)] text: String,
        #[prop(default = 0)] _something_else: usize,
    ) -> impl Render {
        let text = StoredValue::new(text);
        view! {
            <List>
                <>
                    <ListItem>{text.get().unwrap()}</ListItem>
                </>
            </List>
        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(2, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Viewer>
                        "hi"
                    </Viewer>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi"]));
        })
        .await;
}

#[tokio::test]
async fn custom_child_prop() {
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp {
        #[children]
        #[builder(setter(into))]
        text: String,
    }

    #[component]
    fn Viewer(#[prop(into, children)] children: ChildProp) -> impl Render {
        let text = StoredValue::new(children.text);
        view! {
            <List>
                <>
                    <ListItem>{text.get().unwrap()}</ListItem>
                </>
            </List>

        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(2, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Viewer>
                        <ChildProp>{"hi"}</ChildProp>
                    </Viewer>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi"]));
        })
        .await;
}

#[tokio::test]
async fn component_child() {
    #[component]
    fn Viewer<V: Render + 'static>(#[prop(children)] children: V) -> impl Render {
        view! {
            <Col>
                {children}
            </Col>
        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(2, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
                    <Viewer>
                        {view! {
                            <List>
                                <>
                                    <ListItem>{"hi"}</ListItem>
                                </>
                            </List>
                        }}
                    </Viewer>
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi"]));
        })
        .await;
}

#[tokio::test]
async fn component_child_nested() {
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp<V: Render> {
        #[children]
        views: V,
    }

    #[component]
    fn Viewer<V: Render + 'static>(#[prop(children)] children: ChildProp<V>) -> impl Render {
        view! {
            <Col>
                {children.views}
            </Col>
        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(3, 2);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
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
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi", "bye"]));
        })
        .await;
}

#[tokio::test]
async fn custom_component_nested_layout() {
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp<V: Render> {
        #[children]
        views: V,
    }

    #[component]
    fn Viewer<V: Render + 'static>(#[prop(children)] children: ChildProp<V>) -> impl Render {
        view! {
            <Col>
                {children.views}
            </Col>
        }
    }

    LocalSet::new()
        .run_until(async move {
            let backend = TestBackend::new(2, 1);
            let mut terminal = Terminal::new(backend).unwrap();

            let view = view! {
                <Col>
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
                </Col>
            };
            let (tx, rx) = watch::channel(());
            mount(|| view, tx);
            terminal
                .draw(|f: &mut Frame| {
                    render_dom(f);
                })
                .unwrap();

            terminal
                .backend()
                .assert_buffer(&Buffer::with_lines(vec!["hi"]));
        })
        .await;
}
