// use std::backtrace::Backtrace;
// use std::cell::RefCell;
// use std::error::Error;
// use std::io::{stdout, Stdout};
// use std::time::Duration;

// use crossterm::event::{self, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind};
// use crossterm::execute;
// use crossterm::terminal::{
//     disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
// };
// use leptos_reactive::{
//     create_runtime, create_rw_signal, create_signal, RwSignal, Signal, SignalGet,
//     SignalGetUntracked, SignalUpdate,
// };
// use ratatui::backend::CrosstermBackend;
// use rooibos_dom::prelude::*;
// type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

// thread_local! {
//     static KEY_HANDLERS: RefCell<Vec<Box<dyn Fn(KeyEvent)>>> = RefCell::new(vec![]);
// }

// type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {}

// fn main() -> Result<()> {
//     let _ = create_runtime();
//     let mut terminal = setup_terminal()?;
//     mount(view!(<App/>));

//     std::panic::set_hook(Box::new(|panic_info| {
//         crossterm::execute!(std::io::stderr(),
// crossterm::terminal::LeaveAlternateScreen).unwrap();
//         crossterm::terminal::disable_raw_mode().unwrap();
//         let backtrace = Backtrace::capture();
//         println!("{panic_info} {backtrace}");
//     }));

//     terminal.draw(|f: &mut Frame| {
//         render_dom(f);
//     })?;
//     loop {
//         let e = handle_events()?;
//         if e == 0 {
//             restore_terminal(terminal)?;
//             return Ok(());
//         }
//         if e == 1 {
//             terminal.draw(|f: &mut Frame| {
//                 render_dom(f);
//             })?;
//         }
//     }
// }

// fn setup_terminal() -> Result<Terminal> {
//     enable_raw_mode()?;
//     let mut stdout = stdout();
//     execute!(stdout, EnterAlternateScreen)?;
//     let backend = CrosstermBackend::new(stdout);
//     let terminal = Terminal::new(backend)?;
//     Ok(terminal)
// }

// fn restore_terminal(mut terminal: Terminal) -> Result<()> {
//     disable_raw_mode()?;
//     execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
//     Ok(())
// }

// fn handle_events() -> Result<usize> {
//     if event::poll(Duration::from_millis(100))? {
//         if let Event::Key(key) = event::read()? {
//             if let KeyCode::Char('q') = key.code {
//                 return Ok(0);
//             }
//             KEY_HANDLERS.with(|h| h.borrow().iter().for_each(|h| (h)(key)));
//         }
//     }
//     Ok(2)
// }

// #[component]
// fn App() -> impl IntoView {
//     let child2_id = create_rw_signal(0);

//     view! {
//         <Router initial="/">
//             <Route path="/">
//                 {move || view!(<Child0/>)}
//             </Route>
//             <Route path="/child1">
//                 {move || view!(<Child1 child2_id=child2_id/>)}
//             </Route>
//             <Route path="/child2/:id">
//                 {move || view!(<Child2/>)}
//             </Route>
//         </Router>
//     }
// }

// #[component]
// fn Child0() -> impl IntoView {
//     let router = use_router();
//     KEY_HANDLERS.with(|h| {
//         h.borrow_mut().push(Box::new(move |event| {
//             if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
//                 router.push("/child1?id=1");
//             }
//         }))
//     });

//     view! {
//         <Paragraph>
//             "child0"
//         </Paragraph>
//     }
// }

// #[component]
// fn Child1(child2_id: RwSignal<i32>) -> impl IntoView {
//     let router = use_router();
//     let id = router.use_query("id");
//     // let context = use_event_context(cx);

//     KEY_HANDLERS.with(|h| {
//         h.borrow_mut().push(Box::new(move |event| {
//             if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
//                 router.push(format!("/child2/{}", child2_id.get_untracked()));
//                 child2_id.update(|id| *id += 1);
//             }
//         }))
//     });

//     view! {
//         <Paragraph>
//             {format!("child1 id={}", id.get().unwrap())}
//         </Paragraph>
//     }
// }

// #[component]
// fn Child2() -> impl IntoView {
//     let router = use_router();
//     let id = router.use_param("id");

//     KEY_HANDLERS.with(|h| {
//         h.borrow_mut().push(Box::new(move |event| {
//             if event.code == KeyCode::Enter && event.kind == KeyEventKind::Press {
//                 router.pop();
//             }
//         }))
//     });

//     view! {
//         <Paragraph>
//             {format!("child2 id={}", id.get().unwrap())}
//         </Paragraph>
//     }
// }
