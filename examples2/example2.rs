use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use once_cell::sync::Lazy;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Block,
    Frame, Terminal,
};
use rooibos::{
    reactive::{create_child_scope, create_signal, Scope, SignalGet, SignalUpdate},
    run_system, use_event_context, EventHandler,
};
use std::{
    error::Error,
    io::{stdout, Stdout},
};
use tui_rsx::{prelude::*, typemap::TypeMap};
fn main() -> Result<(), Box<dyn Error>> {
    run_system(run)
}

fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
    let body = async {
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        let handler = EventHandler::initialize(cx, terminal);
        let mut v = {
            let __parent_id = 0;
            let mut __fn1 = ::std::rc::Rc::new(::std::cell::RefCell::new(Counter(
                create_child_scope(cx),
                CounterProps::builder().__caller_id(0u32).build(),
            )));
            move |f: &mut Frame<_>, rect: Rect| {
                let mut __fn1 = __fn1.clone();
                let layout = Layout::default().direction(Direction::Vertical);
                let chunks = layout.constraints([Constraint::Length(5)]).split(rect);
                (__fn1).view(f, chunks[0usize]);
            }
        };
        handler.render(move |terminal| {
            terminal
                .draw(|f| {
                    v.view(f, f.size());
                })
                .unwrap();
        });
        let mut terminal = handler.run().await;
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )
        .unwrap();
        terminal.show_cursor().unwrap();
        Ok(())
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}

#[derive(typed_builder::TypedBuilder)]
struct CounterProps<B: Backend + 'static> {
    #[builder(setter(doc = "**_phantom**: [`::std::marker::PhantomData<(B)>`]"))]
    #[builder(default_code = "Default :: default()")]
    _phantom: ::std::marker::PhantomData<(B)>,
    __caller_id: u32,
}

thread_local! {
    static __COUNTER_CACHE:
        ::std::cell::RefCell<tui_rsx::once_cell::sync::Lazy<tui_rsx::typemap::TypeMap>> = std::cell::RefCell::new(Lazy::new(TypeMap::new));
}

#[allow(non_snake_case, clippy::too_many_arguments, unused_mut)]
fn Counter<B: Backend + 'static>(
    #[allow(unused_variables)] cx: Scope,
    props: CounterProps<B>,
) -> impl View<B> {
    fn __Counter<B: Backend + 'static>(cx: Scope, __parent_id: u32) -> impl LazyView<B> {
        let count = create_signal(cx, 0);
        let context = use_event_context(cx);
        context.create_key_effect(cx, move |event| {
            if event.code == KeyCode::Enter {
                count.update(|c| *c += 1);
            }
        });
        move || {
            let mut __fn3 = ::std::rc::Rc::new(::std::cell::RefCell::new(block(
                create_child_scope(cx),
                BlockProps::builder()
                    .__caller_id(
                        (__parent_id.to_string() + &2u32.to_string())
                            .parse()
                            .expect("invalid integer"),
                    )
                    .title({
                        let res = format!("count {0}", count.get());
                        res
                    })
                    .build(),
            )));
            (__fn3)
        }
    }
    let CounterProps {
        _phantom,
        __caller_id,
    } = props;
    __COUNTER_CACHE.with(|c| {
        let mut cache_mut = c.borrow_mut();
        if let Some(map) = cache_mut.get_mut::<tui_rsx::KeyWrapper<B>>() {
            if let Some(cache) = map.get(&__caller_id) {
                cache.clone()
            } else {
                let res = ::std::rc::Rc::new(::std::cell::RefCell::new(
                    tui_rsx::LazyViewWrapper::new(__Counter(create_child_scope(cx), __caller_id)),
                ));
                map.insert(__caller_id, res.clone());
                res
            }
        } else {
            let mut map = ::std::collections::HashMap::<
                u32,
                ::std::rc::Rc<::std::cell::RefCell<dyn View<B>>>,
            >::new();
            let res = ::std::rc::Rc::new(::std::cell::RefCell::new(tui_rsx::LazyViewWrapper::new(
                __Counter(create_child_scope(cx), __caller_id),
            )));
            map.insert(__caller_id, res.clone());
            cache_mut.insert::<tui_rsx::KeyWrapper<B>>(map);
            res
        }
    })
}
