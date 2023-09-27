use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};
use ratatui::Terminal;
use reactive::create_root;
use rooibos_reactive::Scope;
use rooibos_rsx::prelude::*;
use rooibos_rsx::view;
use typed_builder::TypedBuilder;

#[test]
fn standalone_widget() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Block title="test" borders=Borders::ALL/>
        };

        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();

        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "┌test────┐",
            "│        │",
            "└────────┘",
        ]));
    });
}

#[test]
fn widget_no_props() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Block/>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();

        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "          ",
            "          ",
            "          ",
        ]));
    });
}

#[test]
fn simple_column() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Block title="test" borders=Borders::ALL/>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();

        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "┌test────┐",
            "│        │",
            "└────────┘",
        ]));
    });
}

#[test]
fn nested_layout() {
    let backend = TestBackend::new(10, 6);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Row v:length=4>
                    <Block title="test1" borders=Borders::ALL/>
                </Row>
                <Row v:length=2>
                    <Block title="test2" borders=Borders::ALL/>
                </Row>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
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
    });
}

#[test]
fn conditional() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let a = 1;
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                {
                    match a {
                        1 => view!(cx, <Block title="test" borders=Borders::ALL/>),
                        _ => view!(cx, <Block title="test2" borders=Borders::ALL/>)
                    }
                }
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();

        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "┌test────┐",
            "│        │",
            "└────────┘",
        ]));
    });
}

#[test]
fn list_basic() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <List>
                    <ListItem>"test1"</ListItem>
                    <ListItem>"test2"</ListItem>
                </List>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();

        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "test1     ",
            "test2     ",
            "          ",
        ]));
    });
}

#[test]
fn prop_iteration() {
    let backend = TestBackend::new(10, 6);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <List>
                    {
                        (0..5).map(|i| prop!(<ListItem>{format!("test{i}")}</ListItem>))
                            .collect::<Vec<_>>()
                    }
                </List>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
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
    });
}

#[test]
fn stateful() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let state = ListState::default();
        let mut view = mount! { cx,
            <StatefulList v:state=state>
                <ListItem>"test1"</ListItem>
                <ListItem>"test2"</ListItem>
            </StatefulList>
        };

        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();

        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "test1     ",
            "test2     ",
            "          ",
        ]));
    });
}

#[test]
fn list_styled() {
    let backend = TestBackend::new(15, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <List>
                    <ListItem style=prop!(<Style fg=Color::Black/>)>"test1"</ListItem>
                    <ListItem>"test2"</ListItem>
                </List>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
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
    });
}

#[test]
fn block_children() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Tabs>
                    "tab1"
                    "tab2"
                </Tabs>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
    });
}

#[test]
fn single_child_as_vec() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Tabs>
                    <>{"tab1"}</>
                </Tabs>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
    });
}

#[test]
fn single_nested_child_as_vec() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
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

        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
    });
}

#[test]
fn complex_block_children() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Tabs select=0>
                    <Line>"tab1"</Line>
                    <Line>{vec![Span::from("tab2")]}</Line>
                </Tabs>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
    });
}

#[test]
fn macro_as_prop() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Paragraph block=prop!{<Block borders=Borders::ALL/>}>
                    "test"
                </Paragraph>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "┌────────┐",
            "│test    │",
            "└────────┘",
        ]));
    });
}

#[test]
fn simple_overlay() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Overlay>
                <Block borders=Borders::ALL/>
                <Paragraph>
                    "test"
                </Paragraph>
            </Overlay>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&Buffer::with_lines(vec![
            "test─────┐",
            "│        │",
            "└────────┘",
        ]));
    });
}

#[test]
fn overlay_multiple() {
    let backend = TestBackend::new(10, 6);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
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
        terminal
            .draw(|f| {
                view.view(f, f.size());
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
    });
}

#[test]
fn two_overlays() {
    let backend = TestBackend::new(10, 8);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
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
        terminal
            .draw(|f| {
                view.view(f, f.size());
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
    });
}

#[test]
fn array_as_variable() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let tab_items = vec!["tab1", "tab2"];
        let mut view = mount! { cx,
            <Column>
                <Tabs>
                    {tab_items}
                </Tabs>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
    });
}

#[test]
fn simple_custom_component() {
    #[component]
    fn Viewer(cx: Scope, #[prop(into)] text: String, flag: bool) -> impl View {
        move || {
            view! { cx,
                <List>
                    <>
                        <ListItem>{format!("{text}={flag}")}</ListItem>
                    </>
                </List>
            }
        }
    }

    let backend = TestBackend::new(7, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Viewer text="hi" flag=true/>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi=true"]));
    });
}

#[test]
fn custom_component_children() {
    #[component]
    fn Viewer(cx: Scope, #[prop(into, children)] text: String) -> impl View {
        move || {
            view! { cx,
                <List>
                    <>
                        <ListItem>{text.clone()}</ListItem>
                    </>
                </List>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Viewer>
                    "hi"
                </Viewer>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi"]));
    });
}

#[test]
fn generic_component() {
    #[component]
    fn Viewer<T: 'static>(cx: Scope, #[prop(into)] text: String, flag: bool) -> impl View {
        let _ = std::any::type_name::<T>();
        move || {
            view! { cx,
                <List>
                    <>
                        <ListItem>{format!("{text}={flag}")}</ListItem>
                    </>
                </List>
            }
        }
    }

    let backend = TestBackend::new(7, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Viewer<usize> text="hi" flag=true/>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi=true"]));
    });
}

#[test]
fn custom_component_children_second() {
    #[component]
    fn Viewer(
        cx: Scope,
        #[prop(default = 0)] _something: usize,
        #[prop(into, children)] text: String,
        #[prop(default = 0)] _something_else: usize,
    ) -> impl View {
        move || {
            view! { cx,
                <List>
                    <>
                        <ListItem>{text.clone()}</ListItem>
                    </>
                </List>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Viewer>
                    "hi"
                </Viewer>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi"]));
    });
}

#[test]
fn custom_child_prop() {
    #[caller_id]
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp {
        #[children]
        #[builder(setter(into))]
        text: String,
    }

    #[component]
    fn Viewer(cx: Scope, #[prop(into, children)] children: ChildProp) -> impl View {
        move || {
            view! { cx,
                <List>
                    <>
                        <ListItem>{children.text.clone()}</ListItem>
                    </>
                </List>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Viewer>
                    <ChildProp>{"hi"}</ChildProp>
                </Viewer>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi"]));
    });
}

#[test]
fn component_child() {
    #[component]
    fn Viewer<V: LazyView + Clone>(_cx: Scope, #[prop(children)] children: V) -> impl View {
        move || {
            let mut children = children.clone();
            view! { cx,
                <Column>
                    {children}
                </Column>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Viewer> {
                    move || view! { cx,
                        <List>
                            <>
                                <ListItem>{"hi"}</ListItem>
                            </>
                        </List>
                    }
                }
                </Viewer>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi"]));
    });
}

#[test]
fn component_child_nested() {
    #[caller_id]
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp<V: LazyView + Clone> {
        #[children]
        views: V,
    }

    #[component]
    fn Viewer<V: LazyView + Clone>(
        _cx: Scope,
        #[prop(children)] children: ChildProp<V>,
    ) -> impl View {
        move || {
            let mut children = children.views.clone();
            view! { cx,
                <Column>
                    {children}
                </Column>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Viewer>
                    <ChildProp> {
                        move || view! { cx,
                            <List>
                                <>
                                    <ListItem>{"hi"}</ListItem>
                                </>
                            </List>
                        }
                    }
                    </ChildProp>
                </Viewer>
            </Column>
        };
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi"]));
    });
}

#[test]
fn custom_component_nested_layout() {
    #[caller_id]
    #[derive(TypedBuilder, ComponentChildren)]
    struct ChildProp<V: LazyView + Clone> {
        #[children]
        views: V,
    }

    #[component]
    fn Viewer<V: LazyView + Clone>(
        _cx: Scope,
        #[prop(children)] children: ChildProp<V>,
    ) -> impl View {
        move || {
            let mut children = children.views.clone();
            view! { cx,
                <Column>
                    {children}
                </Column>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <Column>
                <Row v:length=1>
                    <Viewer>
                        <ChildProp> {
                            move || view! { cx,
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
        terminal
            .draw(|f| {
                view.view(f, f.size());
            })
            .unwrap();
        terminal
            .backend()
            .assert_buffer(&Buffer::with_lines(vec!["hi"]));
    });
}
