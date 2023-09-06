use std::marker::PhantomData;

use ratatui::backend::{Backend, TestBackend};
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
            <block title="test" borders=Borders::ALL/>
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
            <column>
                <block/>
            </column>
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
            <column>
                <block title="test" borders=Borders::ALL/>
            </column>
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
            <column>
                <row length=4>
                    <block title="test1" borders=Borders::ALL/>
                </row>
                <row length=2>
                    <block title="test2" borders=Borders::ALL/>
                </row>
            </column>
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
            <column>
                {
                    match a {
                        1 => view!(cx, <block title="test" borders=Borders::ALL/>),
                        _ => view!(cx, <block title="test2" borders=Borders::ALL/>)
                    }
                }
            </column>
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
            <column>
                <list>
                    <listItem>"test1"</listItem>
                    <listItem>"test2"</listItem>
                </list>
            </column>
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
            <column>
                <list>
                    {
                        (0..5).map(|i| prop!(<listItem>{format!("test{i}")}</listItem>))
                            .collect::<Vec<_>>()
                    }
                </list>
            </column>
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
            <stateful_list state=state>
                <listItem>"test1"</listItem>
                <listItem>"test2"</listItem>
            </stateful_list>
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
            <column>
                <list>
                    <listItem style=Style::default().fg(Color::Black)>"test1"</listItem>
                    <listItem>"test2"</listItem>
                </list>
            </column>
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
            <column>
                <tabs>
                    "tab1"
                    "tab2"
                </tabs>
            </column>
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
            <column>
                <tabs>
                    <>{"tab1"}</>
                </tabs>
            </column>
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
            <column>
                <tabs>
                    <>
                        <line>
                            <span>"tab1"</span>
                        </line>
                    </>
                </tabs>
            </column>
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
            <column>
                <tabs select=0>
                    <line>"tab1"</line>
                    <line>{vec![Span::from("tab2")]}</line>
                </tabs>
            </column>
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
            <column>
                <paragraph block=prop!{<block borders=Borders::ALL/>}>
                    "test"
                </paragraph>
            </column>
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
            <overlay>
                <block borders=Borders::ALL/>
                <paragraph>
                    "test"
                </paragraph>
            </overlay>
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
            <overlay>
                <block borders=Borders::ALL title="test"/>
                <column margin=1>
                    <list length=2>
                        <listItem>{"hi"}</listItem>
                        <listItem>{"yo"}</listItem>
                    </list>
                    <list>
                        <listItem>{"hi2"}</listItem>
                        <listItem>{"yo2"}</listItem>
                    </list>
                </column>
            </overlay>
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
            <column>
                <column percentage=50>
                    <overlay>
                        <block borders=Borders::ALL title="test"/>
                        <column margin=1>
                            <list length=2>
                                <listItem>{"hi"}</listItem>
                                <listItem>{"yo"}</listItem>
                            </list>
                        </column>
                    </overlay>
                </column>
                <column percentage=50>
                    <overlay>
                        <block borders=Borders::ALL title="test2"/>
                        <column margin=1>
                            <list length=2>
                                <listItem>{"hi2"}</listItem>
                                <listItem>{"yo2"}</listItem>
                            </list>
                        </column>
                    </overlay>
                </column>
            </column>
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
            <column>
                <tabs>
                    {tab_items}
                </tabs>
            </column>
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
    fn Viewer<B: Backend + 'static>(
        cx: Scope,
        #[prop(into)] text: String,
        flag: bool,
    ) -> impl View<B> {
        move || {
            view! { cx,
                <list>
                    <>
                        <listItem>{format!("{text}={flag}")}</listItem>
                    </>
                </list>
            }
        }
    }

    let backend = TestBackend::new(7, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <Viewer text="hi" flag/>
            </column>
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
fn static_backend_component() {
    #[component]
    fn Viewer(cx: Scope, #[prop(into)] text: String) -> impl View<TestBackend> {
        move || {
            view! { cx,
                <list>
                    <>
                        <listItem>{text.clone()}</listItem>
                    </>
                </list>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <Viewer text="hi"/>
            </column>
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
fn custom_component_children() {
    #[component]
    fn Viewer<B: Backend + 'static>(
        cx: Scope,
        #[prop(into, children)] text: String,
    ) -> impl View<B> {
        move || {
            view! { cx,
                <list>
                    <>
                        <listItem>{text.clone()}</listItem>
                    </>
                </list>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <Viewer>
                    "hi"
                </Viewer>
            </column>
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
fn custom_component_children_second() {
    #[component]
    fn Viewer<B: Backend + 'static>(
        cx: Scope,
        #[prop(default = 0)] _something: usize,
        #[prop(into, children)] text: String,
        #[prop(default = 0)] _something_else: usize,
    ) -> impl View<B> {
        move || {
            view! { cx,
                <list>
                    <>
                        <listItem>{text.clone()}</listItem>
                    </>
                </list>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <Viewer>
                    "hi"
                </Viewer>
            </column>
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
    fn Viewer<B: Backend + 'static>(
        cx: Scope,
        #[prop(into, children)] children: ChildProp,
    ) -> impl View<B> {
        move || {
            view! { cx,
                <list>
                    <>
                        <listItem>{children.text.clone()}</listItem>
                    </>
                </list>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <Viewer>
                    <ChildProp>{"hi"}</ChildProp>
                </Viewer>
            </column>
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
    fn Viewer<B: Backend + 'static, V: LazyView<B> + Clone + 'static>(
        _cx: Scope,
        #[prop(children)] children: V,
    ) -> impl View<B> {
        move || {
            let mut children = children.clone();
            view! { cx,
                <column>
                    {children}
                </column>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <Viewer> {
                    move || view! { cx,
                        <list>
                            <>
                                <listItem>{"hi"}</listItem>
                            </>
                        </list>
                    }
                }
                </Viewer>
            </column>
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
    struct ChildProp<B: Backend + 'static, V: LazyView<B> + Clone + 'static> {
        #[children]
        views: V,
        #[builder(default)]
        _phantom: PhantomData<B>,
    }

    #[component]
    fn Viewer<B: Backend + 'static, V: LazyView<B> + Clone + 'static>(
        _cx: Scope,
        #[prop(children)] children: ChildProp<B, V>,
    ) -> impl View<B> {
        move || {
            let mut children = children.views.clone();
            view! { cx,
                <column>
                    {children}
                </column>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <Viewer>
                    <ChildProp> {
                        move || view! { cx,
                            <list>
                                <>
                                    <listItem>{"hi"}</listItem>
                                </>
                            </list>
                        }
                    }
                    </ChildProp>
                </Viewer>
            </column>
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
    struct ChildProp<B: Backend + 'static, V: LazyView<B> + Clone + 'static> {
        #[children]
        views: V,
        #[builder(default)]
        _phantom: PhantomData<B>,
    }

    #[component]
    fn Viewer<B: Backend + 'static, V: LazyView<B> + Clone + 'static>(
        _cx: Scope,
        #[prop(children)] children: ChildProp<B, V>,
    ) -> impl View<B> {
        move || {
            let mut children = children.views.clone();
            view! { cx,
                <column>
                    {children}
                </column>
            }
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    create_root(|cx| {
        let mut view = mount! { cx,
            <column>
                <row length=1>
                    <Viewer>
                        <ChildProp> {
                            move || view! { cx,
                                <list>
                                    <>
                                        <listItem>{"hi"}</listItem>
                                    </>
                                </list>
                            }
                        }
                        </ChildProp>
                    </Viewer>
                </row>
            </column>
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
