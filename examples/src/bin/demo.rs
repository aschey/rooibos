use std::backtrace::Backtrace;
use std::error::Error;
use std::io::stdout;
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind};
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
use rooibos::reactive::{
    create_effect, create_memo, create_selector, create_signal, Scope, Signal, SignalGet,
    SignalUpdate,
};
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

    std::panic::set_hook(Box::new(|panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        let backtrace = Backtrace::capture();
        println!("{panic_info} {backtrace}");
    }));

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
    event_context.create_key_effect(cx, move |event| {
        if event.kind == KeyEventKind::Press {
            match event.code {
                KeyCode::Left => {
                    previous_tab();
                }
                KeyCode::Right => {
                    next_tab();
                }
                _ => {}
            }
        }
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
                    length=3
                    block=prop!(<Block borders=Borders::ALL title="Demo"/>)
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
                        <Tab0 />
                    }}
                </Case>
                <Case when=move || focus_selector.get() == Some(1)>
                    {move || view! { cx,
                        <Tab1 />
                    }}
                </Case>
                <Case when=move || focus_selector.get() == Some(2)>
                    {move || view! { cx,
                        <Tab2 />
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
                <Gauges length=6 enhanced_graphics=true/>
                <Charts enhanced_graphics=true/>
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
struct RandomData<S: Iterator> {
    source: S,
    pub points: Vec<S::Item>,
    tick_rate: usize,
}

#[derive(Clone)]
pub struct RandomDistribution {
    distribution: Uniform<u64>,
    rng: ThreadRng,
}

impl RandomDistribution {
    pub fn new(lower: u64, upper: u64) -> RandomDistribution {
        RandomDistribution {
            distribution: Uniform::new(lower, upper),
            rng: rand::thread_rng(),
        }
    }
}

impl Iterator for RandomDistribution {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.distribution.sample(&mut self.rng))
    }
}

impl<S> RandomData<S>
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

    create_effect(cx, move || {
        if tick_event.get().is_some() {
            progress.update(|p| {
                if *p < 1.0 {
                    (*p + 0.001f64).min(1.0)
                } else {
                    0.0
                }
            });
        }
    });

    move || {
        view! {cx,
            <Column>
                <DemoGauge length=2 enhanced_graphics=enhanced_graphics progress=progress/>
                <DemoSparkline length=3 enhanced_graphics=enhanced_graphics/>
                <DemoLineGauge length=1 enhanced_graphics=enhanced_graphics progress=progress/>
            </Column>
        }
    }
}

#[component]
fn DemoGauge<B: Backend>(
    cx: Scope,
    enhanced_graphics: bool,
    progress: Signal<f64>,
) -> impl View<B> {
    move || {
        view! { cx,
            <Gauge
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
                ratio=progress.get()
            />
        }
    }
}

#[component]
fn DemoLineGauge<B: Backend>(
    cx: Scope,
    enhanced_graphics: bool,
    progress: Signal<f64>,
) -> impl View<B> {
    move || {
        view! { cx,
            <LineGauge
                block=prop!(<Block title="LineGauge:"/>)
                gauge_style=prop!(<Style fg=Color::Magenta/>)
                line_set=if enhanced_graphics {
                    symbols::line::THICK
                } else {
                    symbols::line::NORMAL
                }
                ratio=progress.get()
        />
        }
    }
}

#[component]
fn DemoSparkline<B: Backend>(cx: Scope, enhanced_graphics: bool) -> impl View<B> {
    let mut rand_signal = RandomDistribution::new(0, 100);
    let sparkline_points = rand_signal.by_ref().take(300).collect();
    let sparkline_signal = create_signal(
        cx,
        RandomData {
            source: rand_signal,
            points: sparkline_points,
            tick_rate: 1,
        },
    );

    let event_context = use_event_context(cx);
    let tick_event = event_context.create_custom_event_signal::<Tick>(cx);

    create_effect(cx, move || {
        if tick_event.get().is_some() {
            sparkline_signal.update(|s| s.on_tick());
        }
    });

    move || {
        view! { cx,
            <Sparkline
                block=prop!(<Block title="Sparkline:"/>)
                style=prop!(<Style fg=Color::Green/>)
                data=sparkline_signal.get().points
                bar_set=if enhanced_graphics {
                    symbols::bar::NINE_LEVELS
                } else {
                    symbols::bar::THREE_LEVELS
                }
        />
        }
    }
}

#[component]
fn Charts<B: Backend>(cx: Scope, enhanced_graphics: bool) -> impl View<B> {
    let show_chart = create_signal(cx, true);

    move || {
        view! { cx,
            <Row>
                <Column percentage=50>
                    <Row percentage=50>
                        <Column percentage=50>
                            <TaskList percentage=50/>
                        </Column>
                        <Column percentage=50>
                            <Logs percentage=50/>
                        </Column>
                    </Row>
                    <Row percentage=50>
                        <DemoBarChart enhanced_graphics=enhanced_graphics/>
                    </Row>
                </Column>
                <Column percentage=50>
                    <DemoChart enhanced_graphics=enhanced_graphics/>
                </Column>
            </Row>
        }
    }
}

const TASKS: [&str; 24] = [
    "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
    "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
    "Item20", "Item21", "Item22", "Item23", "Item24",
];

#[component]
fn TaskList<B: Backend>(cx: Scope) -> impl View<B> {
    let selected_task = create_signal::<Option<usize>>(cx, None);
    let task_data = create_signal(cx, TASKS.to_vec());

    let task_items = create_memo(cx, move || {
        task_data
            .get()
            .into_iter()
            .map(|t| {
                prop! {
                    <ListItem>
                        <>
                            <Line>
                                <Span>{t}</Span>
                            </Line>
                        </>
                    </ListItem>
                }
            })
            .collect::<Vec<_>>()
    });

    move || {
        view! { cx,
            <StatefulList
                state=prop!(<ListState with_selected=selected_task.get()/>)
                block=prop!(<Block borders=Borders::ALL title="List"/>)
                highlight_style=prop!(<Style add_modifier=Modifier::BOLD/>)
                highlight_symbol="> "
            >
                {task_items.get()}
            </StatefulList>
        }
    }
}

const LOGS: [(&str, &str); 26] = [
    ("Event1", "INFO"),
    ("Event2", "INFO"),
    ("Event3", "CRITICAL"),
    ("Event4", "ERROR"),
    ("Event5", "INFO"),
    ("Event6", "INFO"),
    ("Event7", "WARNING"),
    ("Event8", "INFO"),
    ("Event9", "INFO"),
    ("Event10", "INFO"),
    ("Event11", "CRITICAL"),
    ("Event12", "INFO"),
    ("Event13", "INFO"),
    ("Event14", "INFO"),
    ("Event15", "INFO"),
    ("Event16", "INFO"),
    ("Event17", "ERROR"),
    ("Event18", "ERROR"),
    ("Event19", "INFO"),
    ("Event20", "INFO"),
    ("Event21", "WARNING"),
    ("Event22", "INFO"),
    ("Event23", "INFO"),
    ("Event24", "WARNING"),
    ("Event25", "INFO"),
    ("Event26", "INFO"),
];

#[component]
fn Logs<B: Backend>(cx: Scope) -> impl View<B> {
    let log_data = create_signal(cx, LOGS.to_vec());

    let event_context = use_event_context(cx);
    let tick_event = event_context.create_custom_event_signal::<Tick>(cx);

    create_effect(cx, move || {
        if tick_event.get().is_some() {
            log_data.update(|logs| {
                let mut logs = logs.clone();
                let log = logs.pop().unwrap();
                logs.insert(0, log);
                logs
            })
        }
    });

    let logs = create_memo(cx, move || {
        let info_style = Style::default().fg(Color::Blue);
        let warning_style = Style::default().fg(Color::Yellow);
        let error_style = Style::default().fg(Color::Magenta);
        let critical_style = Style::default().fg(Color::Red);

        log_data
            .get()
            .into_iter()
            .map(|(evt, level)| {
                let s = match level {
                    "ERROR" => error_style,
                    "CRITICAL" => critical_style,
                    "WARNING" => warning_style,
                    _ => info_style,
                };
                prop! {
                    <ListItem>
                        <Line>
                            <Span style=s>{format!("{level:<9}")}</Span>
                            <Span>{evt}</Span>
                        </Line>
                    </ListItem>
                }
            })
            .collect::<Vec<_>>()
    });
    move || {
        view! { cx,
            <List
                block=prop!(<Block borders=Borders::ALL title="Logs"/>)
            >
                {logs.get()}
            </List>
        }
    }
}

const EVENTS: [(&str, u64); 24] = [
    ("B1", 9),
    ("B2", 12),
    ("B3", 5),
    ("B4", 8),
    ("B5", 2),
    ("B6", 4),
    ("B7", 5),
    ("B8", 9),
    ("B9", 14),
    ("B10", 15),
    ("B11", 1),
    ("B12", 0),
    ("B13", 4),
    ("B14", 6),
    ("B15", 4),
    ("B16", 6),
    ("B17", 4),
    ("B18", 7),
    ("B19", 13),
    ("B20", 8),
    ("B21", 11),
    ("B22", 9),
    ("B23", 3),
    ("B24", 5),
];

#[component]
fn DemoBarChart<B: Backend>(cx: Scope, enhanced_graphics: bool) -> impl View<B> {
    let barchart_data = create_signal(cx, EVENTS.to_vec());

    let event_context = use_event_context(cx);
    let tick_event = event_context.create_custom_event_signal::<Tick>(cx);

    create_effect(cx, move || {
        if tick_event.get().is_some() {
            barchart_data.update(|data| {
                let mut data = data.clone();
                let event = data.pop().unwrap();
                data.insert(0, event);
                data
            })
        }
    });

    move || {
        view! { cx,
            <BarChart
                block=prop!(<Block borders=Borders::ALL title="Bar chart"/>)
                data=&barchart_data.get()
                bar_width=3
                bar_gap=2
                bar_set=if enhanced_graphics {
                    symbols::bar::NINE_LEVELS
                } else {
                    symbols::bar::THREE_LEVELS
                }
                value_style=prop!{
                    <Style
                        fg=Color::Black
                        bg=Color::Green
                        add_modifier=Modifier::ITALIC/>
                    }
                label_style=prop!(<Style fg=Color::Yellow/>)
                bar_style=prop!(<Style fg=Color::Green/>)
            />
        }
    }
}

#[derive(Clone)]
pub struct SinData {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinData {
    pub fn new(interval: f64, period: f64, scale: f64) -> SinData {
        SinData {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinData {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

#[component]
fn DemoChart<B: Backend>(cx: Scope, enhanced_graphics: bool) -> impl View<B> {
    let mut sin1_data = SinData::new(0.2, 3.0, 18.0);
    let sin1 = create_signal(
        cx,
        RandomData::<SinData> {
            points: sin1_data.by_ref().take(100).collect(),
            source: sin1_data,
            tick_rate: 5,
        },
    );

    let mut sin2_data = SinData::new(0.1, 2.0, 10.0);
    let sin2 = create_signal(
        cx,
        RandomData::<SinData> {
            points: sin2_data.by_ref().take(200).collect(),
            source: sin2_data,
            tick_rate: 10,
        },
    );

    let window = create_signal(cx, [0.0, 20.0]);

    let event_context = use_event_context(cx);
    let tick_event = event_context.create_custom_event_signal::<Tick>(cx);

    create_effect(cx, move || {
        if tick_event.get().is_some() {
            window.update(|[start, end]| [start + 1.0, end + 1.0]);
            sin1.update(|s| s.clone().on_tick());
            sin2.update(|s| s.clone().on_tick());
        }
    });

    move || {
        let [window_start, window_end] = window.get();
        view! { cx,
            <Chart
                block=prop! {
                    <Block
                        title=prop! {
                            <Span style=prop!{
                                <Style
                                    fg=Color::Cyan
                                    add_modifier=Modifier::BOLD/>}
                                >
                                    "Chart"
                                </Span>
                        }
                        borders=Borders::ALL
                    />
                }
                x_axis=prop! {
                    <Axis
                        title="X Axis"
                        style=prop!(<Style fg=Color::Gray/>)
                        bounds=[window_start, window_end]
                        labels=vec![
                            prop! {
                                <Span
                                    style=prop!(<Style add_modifier=Modifier::BOLD/>)
                                >
                                    {window_start.to_string()}
                                </Span>
                            },
                            prop! {
                                <Span>{((window_start + window_end) / 2.0).to_string()}</Span>
                            },
                            prop! {
                                <Span
                                    style=prop!(<Style add_modifier=Modifier::BOLD/>)
                                >
                                    {window_end.to_string()}
                                </Span>
                            },
                        ]
                    />
                }
                y_axis=prop! {
                    <Axis
                        title="Y Axis"
                        style=prop!(<Style fg=Color::Gray/>)
                        bounds=[-20.0, 20.0]
                        labels=vec![
                            prop!(<Span style=prop!(<Style add_modifier=Modifier::BOLD/>)>"-20"</Span>),
                            prop!(<Span>"0"</Span>),
                            prop!(<Span style=prop!(<Style add_modifier=Modifier::BOLD/>)>"20"</Span>)
                        ]
                    />
                }
            >
                <DatasetOwned
                    name="data2"
                    marker=symbols::Marker::Dot
                    style=prop!(<Style fg=Color::Cyan/>)
                    data=sin1.get().points
                />
                <DatasetOwned
                    name="data3"
                    marker=if enhanced_graphics {
                        symbols::Marker::Braille
                    } else {
                        symbols::Marker::Dot
                    }
                    style=prop!(<Style fg=Color::Yellow/>)
                    data=sin2.get().points
                />
            </Chart>
        }
    }
}
