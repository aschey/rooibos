use std::error::Error;
use std::io::stdout;
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use rand::distributions::Uniform;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::{Frame, Terminal};
use rooibos::reactive::{create_effect, create_signal, Scope, SignalGet, SignalUpdate};
use rooibos::rsx::prelude::*;
use rooibos::runtime::{
    provide_focus_context, run_system, use_event_context, use_focus_context, AnyClone, Command,
    EventHandler, Request,
};
use tilia::tower_rpc::transport::ipc::{
    self, IpcSecurity, OnConflict, SecurityAttributes, ServerId,
};
use tilia::tower_rpc::transport::CodecTransport;
use tilia::tower_rpc::LengthDelimitedCodec;
use tokio::time;
use tracing::{info, Level};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

const NUM_TABS: usize = 3;

fn main() -> Result<(), Box<dyn Error>> {
    run_system(run)
}

#[tokio::main]
async fn run(cx: Scope) -> Result<(), Box<dyn Error>> {
    let (ipc_writer, mut guard) = tilia::Writer::new(1024, move || {
        Box::pin(async move {
            let transport = ipc::create_endpoint(
                ServerId("rooibos-demo"),
                SecurityAttributes::allow_everyone_create().unwrap(),
                OnConflict::Overwrite,
            )
            .unwrap();
            CodecTransport::new(transport, LengthDelimitedCodec)
        })
    });

    tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env()
                .add_directive(Level::TRACE.into())
                .add_directive("tokio_util=info".parse().unwrap())
                .add_directive("tokio_tower=info".parse().unwrap()),
        )
        .with({
            Layer::new()
                .compact()
                .with_writer(ipc_writer)
                .with_filter(tilia::Filter::default())
        })
        .init();

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let handler = EventHandler::initialize(cx, terminal);

    handler.render(mount! { cx,
        <App/>
    });

    info!("Starting");

    let mut terminal = handler.run().await;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    guard.stop().await.ok();
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Tick;

#[component]
fn App<B: Backend>(cx: Scope) -> impl View<B> {
    provide_focus_context::<usize>(cx, Some(0));
    let event_context = use_event_context(cx);
    event_context.dispatch(Command::new_async(|tx, cancellation_token| async move {
        let mut interval = time::interval(Duration::from_millis(250));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    tx.send(Command::simple(Request::Custom(Box::new(Tick))))
                        .await
                        .unwrap();
                }
                _ = cancellation_token.cancelled() => {
                    break;
                }
            }
        }
        None
    }));

    move || {
        view! { cx,
            <Column>
                <HeaderTabs length=3 titles=vec!["Tab0", "Tab1", "Tab2"]/>
            </Column>

        }
    }
}

#[component]
fn HeaderTabs<B: Backend>(cx: Scope, titles: Vec<&'static str>) -> impl View<B> {
    let focus_context = use_focus_context::<usize>(cx);
    let focus_selector = focus_context.get_focus_selector();

    let update_current_tab = move |change: i32| {
        let next = (focus_selector.get().unwrap() as i32 + change).rem_euclid(NUM_TABS as i32);
        focus_context.set_focus(Some(next as usize));
    };

    let previous_tab = move || update_current_tab(-1);
    let next_tab = move || update_current_tab(1);

    let event_context = use_event_context(cx);
    event_context.create_key_effect(cx, move |event| match event.code {
        KeyCode::Left => {
            previous_tab();
        }
        KeyCode::Right => {
            next_tab();
        }
        _ => {}
    });

    move || {
        let titles = titles
            .iter()
            .map(|t| {
                prop! {
                    <Line>
                        <Span style=prop!(<Style fg=Color::Green/>)>
                            {*t}
                        </Span>
                    </Line>
                }
            })
            .collect();
        view! { cx,
            <Column>
                <Tabs
                    length=2
                    block=prop!(<Block borders=Borders::ALL title="demo"/>)
                    highlight_style=prop!(<Style fg=Color::Yellow/>)
                    select=focus_selector.get().unwrap()
                >
                    {titles}
                </Tabs>
                <TabContent/>
            </Column>
        }
    }
}

#[component]
fn TabContent<B: Backend>(cx: Scope) -> impl View<B> {
    let focus_context = use_focus_context::<usize>(cx);
    let focus_selector = focus_context.get_focus_selector();

    move || {
        view! { cx,
            <Switch>
                <Case when=move || focus_selector.get() == Some(0)>
                    {move || view! { cx,
                        <Tab0/>
                    }}
                </Case>
                <Case when=move || focus_selector.get() == Some(1)>
                    {move || view! { cx,
                        <Tab1/>
                    }}
                </Case>
                <Case when=move || focus_selector.get() == Some(2)>
                    {move || view! { cx,
                        <Tab2/>
                    }}
                </Case>
            </Switch>
        }
    }
}

#[component]
fn Tab0<B: Backend>(cx: Scope) -> impl View<B> {
    move || {
        view! {cx,
            <Column>
                <Gauges enhanced_graphics=true/>
            </Column>
        }
    }
}

#[component]
fn Tab1<B: Backend>(cx: Scope) -> impl View<B> {
    move || {
        view! {cx,<Column></Column>}
    }
}

#[component]
fn Tab2<B: Backend>(cx: Scope) -> impl View<B> {
    move || {
        view! {cx,<Column></Column>}
    }
}

#[derive(Clone)]
struct Signal<S: Iterator> {
    source: S,
    pub points: Vec<S::Item>,
    tick_rate: usize,
}

#[derive(Clone)]
pub struct RandomSignal {
    distribution: Uniform<u64>,
    rng: ThreadRng,
}

impl RandomSignal {
    pub fn new(lower: u64, upper: u64) -> RandomSignal {
        RandomSignal {
            distribution: Uniform::new(lower, upper),
            rng: rand::thread_rng(),
        }
    }
}

impl Iterator for RandomSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.distribution.sample(&mut self.rng))
    }
}

impl<S> Signal<S>
where
    S: Iterator + Clone,
    S::Item: Clone,
{
    fn on_tick(&self) -> Self {
        let mut this = self.clone();
        this.points = this.points[this.tick_rate..].to_vec();
        this.points
            .extend(this.source.by_ref().take(this.tick_rate));
        this
    }
}

#[component]
fn Gauges<B: Backend>(cx: Scope, enhanced_graphics: bool) -> impl View<B> {
    let progress = create_signal(cx, 0.0);
    let event_context = use_event_context(cx);
    let tick_event = event_context.create_custom_event_signal::<Tick>(cx);

    let mut rand_signal = RandomSignal::new(0, 100);
    let sparkline_points = rand_signal.by_ref().take(300).collect();

    let sparkline_signal = create_signal(
        cx,
        Signal {
            source: rand_signal,
            points: sparkline_points,
            tick_rate: 1,
        },
    );

    create_effect(cx, move || {
        if tick_event.get().is_some() {
            progress.update(|p| if *p >= 1.0 { 0.0 } else { *p + 0.01 });
            sparkline_signal.update(|s| s.on_tick());
        }
    });

    move || {
        view! {cx,
            <Column>
                <Gauge
                    length=2
                    block=prop!(<Block title="Gauge:"/>)
                    gauge_style=prop!{
                        <Style
                            fg=Color::Magenta
                            bg=Color::Black
                            add_modifier=Modifier::ITALIC | Modifier::BOLD
                        />
                    }
                    use_unicode=enhanced_graphics
                    label=format!("{:.2}%", progress.get() * 100.0)
                />
                <Sparkline
                    length=3
                    block=prop!(<Block title="Sparkline:"/>)
                    style=prop!(<Style fg=Color::Green/>)
                    data=sparkline_signal.get().points
                    bar_set=if enhanced_graphics {
                        symbols::bar::NINE_LEVELS
                    } else {
                        symbols::bar::THREE_LEVELS
                    }
                />
                <LineGauge
                    length=1
                    block=prop!(<Block title="LineGauge:"/>)
                    gauge_style=prop!(<Style fg=Color::Magenta/>)
                    line_set=if enhanced_graphics {
                        symbols::line::THICK
                    } else {
                        symbols::line::NORMAL
                    }
                    ratio=progress.get()
                />
            </Column>
        }
    }
}
