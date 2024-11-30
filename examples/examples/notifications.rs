use std::process::ExitCode;
use std::time::Duration;

use rooibos::components::either_of::Either;
use rooibos::components::spinner::Spinner;
use rooibos::components::{Notification, Notifications, Notifier, provide_notifications};
use rooibos::reactive::dom::layout::{Borders, borders, chars};
use rooibos::reactive::dom::{Render, delay, line, mount, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, height, margin, max_width, padding, row, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize as _;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(CrosstermBackend::stdout())
        .run(app)
        .await
}

fn app() -> impl Render {
    provide_notifications();
    col![
        props(max_width!(100.), borders(Borders::all())),
        (0..5).map(|i| task(i + 1)).collect::<Vec<_>>(),
        Notifications::new().max_layout_width(chars(100.)).render()
    ]
}

fn task(id: usize) -> impl Render {
    let (completed, set_completed) = signal(false);
    let notifier = Notifier::new();
    delay(get_random_delay(), async move {
        set_completed.set(true);
        notifier.notify(Notification::new(format!("task {id} completed")));
    });

    row![move || {
        if completed.get() {
            Either::Left(wgt!(line!("✓ ".green(), span!("task {id}"))))
        } else {
            Either::Right(Spinner::new().label(span!("task {id}")).render())
        }
    }]
}

fn get_random_delay() -> Duration {
    Duration::from_millis(((rand::random::<f32>() * 3. + 1.0) * 1000.0).round() as u64)
}
