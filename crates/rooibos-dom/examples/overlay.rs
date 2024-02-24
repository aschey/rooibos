// use std::cell::{OnceCell, RefCell};
// use std::error::Error;
// use std::io::{stdout, Stdout};
// use std::ops::ControlFlow;
// use std::rc::Rc;
// use std::sync::{Arc, Mutex, OnceLock};
// use std::time::Duration;

// use crossterm::event::{Event, KeyCode};
// use crossterm::terminal::{
//     disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
// };
// use crossterm::{event, execute};
// use leptos_reactive::{
//     create_runtime, create_signal, ReadSignal, Signal, SignalGet, SignalSet, SignalUpdate,
// };
// use ratatui::prelude::{Constraint, CrosstermBackend, Rect};
// use ratatui::widgets::{Borders, Paragraph, Widget};
// use ratatui::Frame;
// use rooibos_dom::{
//     block, mount, paragraph, render_dom, BlockProps, DomNode, IntoView, MakeBuilder,
//     ParagraphProps, View,
// };

// type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
// type Result<T> = std::result::Result<T, Box<dyn Error>>;

// thread_local! {
//     static KEY_HANDLERS: RefCell<Vec<Box<dyn Fn(String)>>> = RefCell::new(vec![]);
// }

fn main() {}

// fn main() -> Result<()> {
//     let _ = create_runtime();
//     let mut terminal = setup_terminal()?;
//     mount(test());
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

// fn handle_events() -> Result<usize> {
//     if event::poll(Duration::from_millis(100))? {
//         if let Event::Key(key) = event::read()? {
//             if let KeyCode::Char('q') = key.code {
//                 return Ok(0);
//             }
//             if let KeyCode::Char(c) = key.code {
//                 KEY_HANDLERS.with(|h| h.borrow().iter().for_each(|h| (h)(c.to_string())));
//                 return Ok(1);
//             }
//         }
//     }
//     Ok(2)
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

// fn test() -> impl IntoView {
//     DomNode::overlay()
//         .child(DomNode::component(block(|| {
//             BlockProps::default().borders(Borders::ALL)
//         })))
//         .child(DomNode::component(paragraph(|| {
//             ParagraphProps::new("test")
//         })))
// }