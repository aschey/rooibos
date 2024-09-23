use std::process::ExitCode;
use std::time::Duration;

use rooibos::components::either_of::Either;
use rooibos::components::spinner::Spinner;
use rooibos::components::{Notification, Notifications, Notifier};
use rooibos::dom::{delay, line, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::layout::{block, chars};
use rooibos::reactive::{Render, col, height, max_width, mount, row, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize as _;
use rooibos::tui::widgets::Block;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    col![
        props(max_width!(100.), block(Block::bordered())),
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

    row![props(height!(1.)), move || {
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
