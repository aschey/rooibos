use std::process::ExitCode;
use std::time::Duration;

use rooibos::components::either_of::Either;
use rooibos::components::spinner::Spinner;
use rooibos::components::{Notification, Notifications, Notifier};
use rooibos::reactive::dom::layout::{Borders, borders, full, height, width};
use rooibos::reactive::dom::{Render, RenderAny, line, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, delay, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, max_viewport_width};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize as _;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?).run(app).await
}

fn app() -> impl Render {
    max_viewport_width(100).unwrap();

    let notifier = Notifier::new();
    col![
        style(width(full()), height(full()), borders(Borders::all())),
        (0..5).map(|i| task(i + 1, notifier)).collect::<Vec<_>>(),
        Notifications::new().render()
    ]
}

fn task(id: usize, notifier: Notifier) -> impl RenderAny {
    let (completed, set_completed) = signal(false);

    delay(get_random_delay(), async move {
        set_completed.set(true);
        notifier.notify(Notification::new(span!("task {id} completed")));
    });

    move || {
        if completed.get() {
            Either::Left(wgt!(line!("✓ ".green(), span!("task {id}"))))
        } else {
            Either::Right(Spinner::new().label(span!("task {id}")).render())
        }
    }
}

fn get_random_delay() -> Duration {
    Duration::from_millis(((rand::random::<f32>() * 3. + 1.0) * 1000.0).round() as u64)
}
