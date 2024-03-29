// TODO re-enable when ratatui gets updated

fn main() {}

// use std::error::Error;
// use std::io::stdout;

// use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
// use crossterm::execute;
// use crossterm::terminal::{
//     disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
// };
// use ratatui::backend::CrosstermBackend;
// use ratatui::{Frame, Terminal};
// use ratatui_textarea::TextArea;
// use rooibos_old::reactive::{create_signal, Scope, Signal, SignalGet, SignalUpdate};
// use rooibos_old::rsx::prelude::*;
// use rooibos_old::rsx::BuilderFacade;
// use rooibos_old::runtime::{run_system, use_event_context, EventHandler};

// fn main() -> Result<(), Box<dyn Error>> {
//     run_system(run)
// }

// #[tokio::main]
// async fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
//     enable_raw_mode()?;
//     let mut stdout = stdout();
//     execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//     let backend = CrosstermBackend::new(stdout);
//     let terminal = Terminal::new(backend)?;

//     let handler = EventHandler::initialize(cx, terminal);

//     handler.render(mount! { cx,
//         <TextView/>
//     });

//     let mut terminal = handler.run().await;
//     disable_raw_mode()?;
//     execute!(
//         terminal.backend_mut(),
//         LeaveAlternateScreen,
//         DisableMouseCapture,
//     )?;
//     terminal.show_cursor()?;
//     Ok(())
// }

// #[component]
// fn TextView(cx: Scope) -> impl View {
//     let event_context = use_event_context(cx);

//     let mut text_area = TextArea::default();
//     text_area.set_block(prop!(<Block borders=Borders::ALL title="Example"/>));
//     let text_area = create_signal(cx, text_area);

//     event_context.create_key_effect(cx, move |event| {
//         text_area.update(|mut t| {
//             t.input(event);
//             t
//         });
//     });
//     move || {
//         view! { cx,
//             <TextAreaWidget text_area=text_area/>
//         }
//     }
// }

// #[component]
// fn TextAreaWidget(_cx: Scope, text_area: Signal<TextArea<'static>>) -> impl View {
//     move || move |f: &mut Frame, area: Rect| f.render_widget(text_area.get().widget(), area)
// }
