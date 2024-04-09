use std::backtrace::Backtrace;
use std::error::Error;
use std::io::{stdout, Stdout};
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use rand::distributions::Uniform;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::canvas::{self, Circle, Context, Map, MapResolution, Rectangle};
use ratatui::Frame;
use rooibos::dom::{component, mount, render_dom, view};
use rooibos::prelude::*;
use rooibos::reactive::{
    create_effect, create_memo, create_rw_signal, create_signal, ReadSignal, RwSignal, Signal,
    SignalGet, SignalUpdate,
};
use rooibos::runtime::{Runtime, TickResult};
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

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

const NUM_TABS: usize = 3;

#[tokio::main]
async fn main() -> Result<()> {
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

    let mut terminal = setup_terminal()?;

    std::panic::set_hook(Box::new(|panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        let backtrace = Backtrace::capture();
        println!("{panic_info} {backtrace}");
    }));

    let mut rt = Runtime::initialize();

    mount(|| view!(<App/>));
    // print_dom(&mut std::io::stdout(), true);
    // Ok(())
    terminal.draw(|f: &mut Frame| {
        render_dom(f);
    })?;

    loop {
        if rt.tick().await == TickResult::Exit {
            restore_terminal(terminal)?;
            guard.stop().await.ok();
            return Ok(());
        }
        terminal.draw(|f: &mut Frame| {
            render_dom(f);
        })?;
    }
}

fn setup_terminal() -> Result<Terminal> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Tick;

#[component]
fn App() -> impl IntoView {
    let event_context = use_event_context();
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

    view! {
        <Col v:length=3>
            <HeaderTabs titles=vec!["Tab0", "Tab1", "Tab2"]/>
        </Col>

    }
}

#[component]
fn HeaderTabs(titles: Vec<&'static str>) -> impl IntoView {
    let focus_context = use_focus_context::<usize>();
    let focus_selector = focus_context.get_focus_selector();

    let update_current_tab = move |change: i32| {
        let next = (focus_selector.get().unwrap() as i32 + change).rem_euclid(NUM_TABS as i32);
        focus_context.set_focus(Some(next as usize));
    };

    let previous_tab = move || update_current_tab(-1);
    let next_tab = move || update_current_tab(1);

    let event_context = use_event_context();
    event_context.create_key_effect(move |event| {
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

    let titles = titles.iter().map(|t| {
        prop! {
            <Line>
                <Span green>
                    {*t}
                </Span>
            </Line>
        }
    });
    view! {
        <Col>
            <Tabs
                v:length=3
                block=prop!(<Block borders=Borders::ALL title="Demo"/>)
                highlight_style=prop!(<Style yellow/>)
                select=focus_selector.get().unwrap()
            >
                {titles}
            </Tabs>
            <TabContent/>
        </Col>
    }
}

#[component]
fn TabContent() -> impl IntoView {
    let focus_context = use_focus_context::<usize>();
    let focus_selector = focus_context.get_focus_selector();

    view! {
        <Switch>
            <Case when=move || focus_selector.get() == Some(0)>
                {move || view! ( <Tab0 />)}
            </Case>
            <Case when=move || focus_selector.get() == Some(1)>
                {move || view! ( <Tab1 />)}
            </Case>
            <Case when=move || focus_selector.get() == Some(2)>
                {move || view! ( <Tab2 />)}
            </Case>
        </Switch>
    }
}

#[component]
fn Tab0() -> impl IntoView {
    view! {
        <Col>
            <Gauges constraint=Constraint::Length(9) enhanced_graphics=true/>
            <Charts constraint=Constraint::Min(8) enhanced_graphics=true/>
            <Footer constraint=Constraint::Length(7) />
        </Col>
    }
}

#[component]
fn Tab1() -> impl IntoView {
    let servers = create_rw_signal(vec![
        Server {
            name: "NorthAmerica-1",
            location: "New York City",
            coords: (40.71, -74.00),
            status: "Up",
        },
        Server {
            name: "Europe-1",
            location: "Paris",
            coords: (48.85, 2.35),
            status: "Failure",
        },
        Server {
            name: "SouthAmerica-1",
            location: "São Paulo",
            coords: (-23.54, -46.62),
            status: "Up",
        },
        Server {
            name: "Asia-1",
            location: "Singapore",
            coords: (1.35, 103.86),
            status: "Up",
        },
    ]);

    view! {
            <Row>
                <DemoTable constraint=Constraint::Percentage(30) servers=servers/>
                <DemoMap constraint=Constraint::Percentage(70) enhanced_graphics=true
    servers=servers/>         </Row>
        }
}

#[component]
fn Tab2() -> impl IntoView {
    view! {
        <Row>
            <ColorsTable constraint=Constraint::Ratio(2, 1)/>
            <Col v:ratio=(2,1)/>
        </Row>
    }
}

#[derive(Clone)]
struct RandomData<S>
where
    S: Iterator + Clone,
    S::Item: Clone,
{
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
    fn on_tick(mut self) -> Self {
        self.points = self.points[self.tick_rate..].to_vec();
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
        self
    }
}

#[component]
fn Gauges(enhanced_graphics: bool, constraint: Constraint) -> impl IntoView {
    let (progress, set_progress) = create_signal(0.0);
    let event_context = use_event_context();
    let tick_event = event_context.create_custom_event_signal::<Tick>();

    create_effect(move |_| {
        if tick_event.get().is_some() {
            set_progress.update(|p| {
                if p < 1.0 {
                    (p + 0.001f64).min(1.0)
                } else {
                    0.0
                }
            });
        }
    });

    view! {
            <Col v:constraint=constraint>
                <DemoGauge constraint=Constraint::Length(2) enhanced_graphics=enhanced_graphics
    progress=progress/>             <DemoSparkline constraint=Constraint::Length(3)
    enhanced_graphics=enhanced_graphics/>             <DemoLineGauge constraint=Constraint::Length(1)
    enhanced_graphics=enhanced_graphics progress=progress/>         </Col>
        }
}

#[component]
fn DemoGauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    constraint: Constraint,
) -> impl IntoView {
    view! {
        <Gauge
            v:constraint=constraint
            block=prop!(<Block title="Gauge:"/>)
            gauge_style=prop!(<Style magenta on_black italic bold/>)
            use_unicode=enhanced_graphics
            label=format!("{:.2}%", progress.get() * 100.0)
            ratio=progress.get()
        />
    }
}

#[component]
fn DemoLineGauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    constraint: Constraint,
) -> impl IntoView {
    view! {
        <LineGauge
            v:constraint=constraint
            block=prop!(<Block title="LineGauge:"/>)
            gauge_style=prop!(<Style magenta/>)
            line_set=if enhanced_graphics {
                symbols::line::THICK
            } else {
                symbols::line::NORMAL
            }
            ratio=progress.get()
        />
    }
}

#[component]
fn DemoSparkline(enhanced_graphics: bool, constraint: Constraint) -> impl IntoView {
    let mut rand_signal = RandomDistribution::new(0, 100);
    let sparkline_points = rand_signal.by_ref().take(300).collect();
    let sparkline_signal = create_rw_signal(RandomData {
        source: rand_signal,
        points: sparkline_points,
        tick_rate: 1,
    });

    let event_context = use_event_context();
    let tick_event = event_context.create_custom_event_signal::<Tick>();

    create_effect(move |_| {
        if tick_event.get().is_some() {
            sparkline_signal.update(|s| s.clone().on_tick());
        }
    });

    view! {
        <Sparkline
            v:constraint=constraint
            block=prop!(<Block title="Sparkline:"/>)
            green
            data=sparkline_signal.get().points
            bar_set=if enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            }
    />
    }
}

#[component]
fn Charts(enhanced_graphics: bool, constraint: Constraint) -> impl IntoView {
    let show_chart = create_signal(true);

    let event_context = use_event_context();
    event_context.create_key_effect(move |event| {
        if event.kind == KeyEventKind::Press && event.code == KeyCode::Char('t') {
            show_chart.toggle();
        }
    });

    view! {
        <Row v:constraint=constraint>
            <Col v:percentage=if show_chart.get() { 50 } else { 100 }>
                <Row v:percentage=50>
                    <Col v:percentage=50>
                        <TaskList constraint=Constraint::Percentage(50)/>
                    </Col>
                    <Col v:percentage=50>
                        <Logs constraint=Constraint::Percentage(50)/>
                    </Col>
                </Row>
                <Row v:percentage=50>
                    <DemoBarChart enhanced_graphics=enhanced_graphics/>
                </Row>
            </Col>
            <Col>
                <DemoChart enhanced_graphics=enhanced_graphics/>
            </Col>
        </Row>
    }
}

const TASKS: [&str; 24] = [
    "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
    "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
    "Item20", "Item21", "Item22", "Item23", "Item24",
];

#[component]
fn TaskList(constraint: Constraint) -> impl IntoView {
    let selected_task = create_rw_signal::<Option<usize>>(None);
    let task_data = create_rw_signal(TASKS.to_vec());

    let task_items = create_memo(move |_| {
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

    view! {
        <StatefulList
            v:constraint=constraint
            v:state=prop!(<ListState with_selected=selected_task.get()/>)
            block=prop!(<Block borders=Borders::ALL title="List"/>)
            highlight_style=prop!(<Style bold/>)
            highlight_symbol="> "
        >
            {task_items.get()}
        </StatefulList>
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
fn Logs(constraint: Constraint) -> impl IntoView {
    let log_data = create_rw_signal(LOGS.to_vec());

    let event_context = use_event_context();
    let tick_event = event_context.create_custom_event_signal::<Tick>();

    create_effect(move |_| {
        if tick_event.get().is_some() {
            log_data.update(|logs| {
                let mut logs = logs.clone();
                let log = logs.pop().unwrap();
                logs.insert(0, log);
                logs
            })
        }
    });

    let logs = create_memo(move |_| {
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

    view! {
        <List
            v:constraint=constraint
            block=prop!(<Block borders=Borders::ALL title="Logs"/>)
        >
            {logs.get()}
        </List>
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
fn DemoBarChart(enhanced_graphics: bool) -> impl IntoView {
    let bar_chart_data = create_rw_signal(EVENTS.to_vec());

    let event_context = use_event_context();
    let tick_event = event_context.create_custom_event_signal::<Tick>();

    create_effect(move |_| {
        if tick_event.get().is_some() {
            bar_chart_data.update(|data| {
                let mut data = data.clone();
                let event = data.pop().unwrap();
                data.insert(0, event);
                data
            })
        }
    });

    view! {
        <BarChart
            block=prop!(<Block borders=Borders::ALL title="Bar chart"/>)
            data=&bar_chart_data.get()
            bar_width=3
            bar_gap=2
            bar_set=if enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            }
            value_style=prop!(<Style black on_green italic/>)
            label_style=prop!(<Style yellow/>)
            bar_style=prop!(<Style green/>)
        />
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
fn DemoChart(enhanced_graphics: bool) -> impl IntoView {
    let mut sin1_data = SinData::new(0.2, 3.0, 18.0);
    let sin1 = create_rw_signal(RandomData::<SinData> {
        points: sin1_data.by_ref().take(100).collect(),
        source: sin1_data,
        tick_rate: 5,
    });

    let mut sin2_data = SinData::new(0.1, 2.0, 10.0);
    let sin2 = create_rw_signal(RandomData::<SinData> {
        points: sin2_data.by_ref().take(200).collect(),
        source: sin2_data,
        tick_rate: 10,
    });

    let window = create_rw_signal([0.0, 20.0]);

    let event_context = use_event_context();
    let tick_event = event_context.create_custom_event_signal::<Tick>();

    create_effect(move |_| {
        if tick_event.get().is_some() {
            window.update(|[start, end]| [start + 1.0, end + 1.0]);
            sin1.update(|s| s.on_tick());
            sin2.update(|s| s.on_tick());
        }
    });

    let window_start = create_memo(move |_| window.get()[0]);
    let window_end = create_memo(move |_| window.get()[1]);

    view! {
            <Chart
                block=prop! {
                    <Block
                        title=prop! {
                            <Span cyan bold>
                                "Chart"
                            </Span>
                        }
                        borders=Borders::ALL
                    />
                }
                x_axis=prop! {
                    <Axis
                        title="X Axis"
                        gray
                        bounds=window.get()
                        labels=vec![
                            prop! {
                                <Span bold>
                                    {window_start.get().to_string()}
                                </Span>
                            },
                            prop! {
                                <Span>{((window_start.get() + window_end.get()) /
    2.0).to_string()}</Span>                         },
                            prop! {
                                <Span bold>
                                    {window_end.get().to_string()}
                                </Span>
                            },
                        ]
                    />
                }
                y_axis=prop! {
                    <Axis
                        title="Y Axis"
                        gray
                        bounds=[-20.0, 20.0]
                        labels=vec![
                            prop!(<Span bold>"-20"</Span>),
                            prop!(<Span>"0"</Span>),
                            prop!(<Span bold>"20"</Span>)
                        ]
                    />
                }
            >
                <DatasetOwned
                    name="data2"
                    marker=symbols::Marker::Dot
                    cyan
                    data=sin1.get().points
                />
                <DatasetOwned
                    name="data3"
                    marker=if enhanced_graphics {
                        symbols::Marker::Braille
                    } else {
                        symbols::Marker::Dot
                    }
                    yellow
                    data=sin2.get().points
                />
            </Chart>
        }
}

#[component]
fn Footer(constraint: Constraint) -> impl IntoView {
    view! {
        <Paragraph
            v:constraint=constraint
            block=prop! {
                <Block
                    borders=Borders::ALL
                    title=prop! {
                        <Span magenta bold>
                            "Footer"
                        </Span>
                    }/>
                }
            wrap=prop!(<Wrap trim=true/>)
            >
            <Line>
                "This is a paragraph with several lines. You can change style your text the way
    you want"             </Line>
            <Line>""</Line>
            <Line>
                <Span>"For example: "</Span>
                <Span red>"under"</Span>
                <Span>" "</Span>
                <Span green>"the"</Span>
                <Span>" "</Span>
                <Span blue>"rainbow"</Span>
                <Span>"."</Span>
            </Line>
            <Line>
                <Span>"Oh and if you didn't "</Span>
                <Span italic>"notice"</Span>
                <Span>" you can "</Span>
                <Span bold>"automatically"</Span>
                <Span>" "</Span>
                <Span reversed>"wrap"</Span>
                <Span>" your "</Span>
                <Span underlined>"text"</Span>
                <Span>"."</Span>
            </Line>
            <Line>
                "One more thing is that it should display unicode characters: 10€"
            </Line>
        </Paragraph>
    }
}

#[derive(Clone)]
pub struct Server<'a> {
    pub name: &'a str,
    pub location: &'a str,
    pub coords: (f64, f64),
    pub status: &'a str,
}

#[component]
fn DemoTable(servers: RwSignal<Vec<Server<'static>>>, constraint: Constraint) -> impl IntoView {
    let rows = create_memo(move |_| {
        servers
            .get()
            .into_iter()
            .map(|s| {
                let style = if s.status == "Up" {
                    prop!(<Style green/>)
                } else {
                    prop!(<Style red rapid_blink crossed_out/>)
                };
                prop!(<Row style=style>{vec![s.name, s.location, s.status]}</Row>)
            })
            .collect::<Vec<_>>()
    });
    view! {
        <Table
            v:constraint=constraint
            header=prop! {
                <Row yellow bottom_margin=1>
                    "Server"
                    "Location"
                    "Status"
                </Row>
            }
            block=prop!(<Block title="Servers" borders=Borders::ALL/>)
        >
            {rows.get()}
            {[
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(10),
            ]}
        </Table>
    }
}

#[component]
fn DemoMap(
    servers: RwSignal<Vec<Server<'static>>>,
    enhanced_graphics: bool,
    constraint: Constraint,
) -> impl IntoView {
    let paint_map = move |ctx: &mut Context<'_>| {
        let servers = servers.get();
        ctx.draw(&Map {
            color: Color::White,
            resolution: MapResolution::High,
        });
        ctx.layer();
        ctx.draw(&Rectangle {
            x: 0.0,
            y: 30.0,
            width: 10.0,
            height: 10.0,
            color: Color::Yellow,
        });
        ctx.draw(&Circle {
            x: servers[2].coords.1,
            y: servers[2].coords.0,
            radius: 10.0,
            color: Color::Green,
        });
        for (i, s1) in servers.iter().enumerate() {
            for s2 in &servers[i + 1..] {
                ctx.draw(&canvas::Line {
                    x1: s1.coords.1,
                    y1: s1.coords.0,
                    y2: s2.coords.0,
                    x2: s2.coords.1,
                    color: Color::Yellow,
                });
            }
        }
        for server in &servers {
            let color = if server.status == "Up" {
                Color::Green
            } else {
                Color::Red
            };
            ctx.print(
                server.coords.1,
                server.coords.0,
                Span::styled("X", Style::default().fg(color)),
            );
        }
    };
    view! {
        <Canvas
            v:constraint=constraint
            block=prop!(<Block title="world" borders=Borders::ALL/>)
            paint=paint_map
            marker=if enhanced_graphics {
                symbols::Marker::Braille
            } else {
                symbols::Marker::Dot
            }
            x_bounds=[-180.0, 180.0]
            y_bounds=[-90.0, 90.0]
        />
    }
}

#[component]
fn ColorsTable(constraint: Constraint) -> impl IntoView {
    let colors = [
        Color::Reset,
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];

    let items = create_signal(
        colors
            .iter()
            .map(|c| {
                prop! {
                    <Row>
                        <Cell>{format!("{c:?}")}</Cell>
                        <Cell fg=*c>"Foreground"</Cell>
                        <Cell bg=*c>"Background"</Cell>
                    </Row>
                }
            })
            .collect::<Vec<_>>(),
    );
    view! {
        <Table
            v:constraint=constraint
            block=prop!(<Block title="Colors" borders=Borders::ALL/>)
        >
            {items.get()}
            {[
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3)
            ]}
        </Table>
    }
}
