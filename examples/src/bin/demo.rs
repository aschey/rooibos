use std::error::Error;
use std::time::Duration;

use crossterm::event::KeyCode;
use rand::distributions::Uniform;
use rand::prelude::*;
use rand::rngs::StdRng;
use rooibos::prelude::canvas::{Circle, Context, Map, MapResolution, Rectangle};
use rooibos::prelude::*;
use rooibos::reactive::computed::Memo;
use rooibos::reactive::effect::Effect;
use rooibos::reactive::owner::{provide_context, use_context, StoredValue};
use rooibos::reactive::signal::{signal, ReadSignal, RwSignal};
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::runtime::{run, use_keypress};
use tilia::tower_rpc::transport::ipc::{
    self, IpcSecurity, OnConflict, SecurityAttributes, ServerId,
};
use tilia::tower_rpc::transport::CodecTransport;
use tilia::tower_rpc::LengthDelimitedCodec;
use tokio::time;
use tracing::Level;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const NUM_TABS: usize = 3;

#[rooibos::main]
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

    mount(|| view!(<App/>));
    run().await?;
    guard.stop().await.unwrap();
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Tick(ReadSignal<u32>);

#[component]
fn App() -> impl Render {
    let (tick, set_tick) = signal(0);
    provide_context(Tick(tick));

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(250));
        let mut seq: u32 = 1;
        loop {
            interval.tick().await;
            set_tick.set(seq);
            seq += 1;
        }
    });

    let titles = StoredValue::new(vec!["Tab0", "Tab1", "Tab2"]);
    view! {
        <Col v:length=3>
            <HeaderTabs titles=titles/>
        </Col>

    }
}

#[component]
fn HeaderTabs(titles: StoredValue<Vec<&'static str>>) -> impl Render {
    let (focused_tab, set_focused_tab) = signal(0);

    let update_current_tab = move |change: i32| {
        set_focused_tab.update(|f| {
            let next = (*f as i32 + change).rem_euclid(NUM_TABS as i32);
            *f = next as usize;
        });
    };

    let previous_tab = move || update_current_tab(-1);
    let next_tab = move || update_current_tab(1);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            match term_signal.code {
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

    view! {
        <Col>
            <Tabs
                v:length=3
                block=prop!(<Block borders=Borders::ALL title="Demo"/>)
                highlight_style=prop!(<Style yellow/>)
                select=focused_tab.get()
            >
                {titles
                    .get()
                    .unwrap()
                    .iter()
                    .map(|t| {
                        prop! {
                            <Line>
                                <Span green>
                                    {*t}
                                </Span>
                            </Line>
                        }
                    })}
            </Tabs>
            <TabContent focused=focused_tab/>
        </Col>
    }
}

#[component]
fn TabContent(focused: ReadSignal<usize>) -> impl Render {
    view! {
        <Row>
            {move || match focused.get() {
                0 => any_view!(<Tab0/>),
                1 => any_view!(<Tab1/>),
                2 => any_view!(<Tab2/>),
                _ => unreachable!()
            }

            }
        </Row>
    }
}

#[component]
fn Tab0() -> impl Render {
    view! {
        <Col>
            <Gauges constraint=Constraint::Length(9) enhanced_graphics=true/>
            <Charts constraint=Constraint::Min(8) enhanced_graphics=true/>
            <Footer constraint=Constraint::Length(7) />
        </Col>
    }
}

#[component]
fn Tab1() -> impl Render {
    let servers = RwSignal::new(vec![
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
            <DemoTable
                constraint=Constraint::Percentage(30)
                servers=servers/>
            <DemoMap
                constraint=Constraint::Percentage(70)
                enhanced_graphics=true
                servers=servers/>
        </Row>
    }
}

#[component]
fn Tab2() -> impl Render {
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
    rng: StdRng,
}

impl RandomDistribution {
    pub fn new(lower: u64, upper: u64) -> RandomDistribution {
        RandomDistribution {
            distribution: Uniform::new(lower, upper),
            rng: rand::rngs::StdRng::from_entropy(),
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
    fn on_tick(&mut self) {
        self.points = self.points[self.tick_rate..].to_vec();
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

#[component]
fn Gauges(enhanced_graphics: bool, constraint: Constraint) -> impl Render {
    let (progress, set_progress) = signal(0.0);

    let tick = use_context::<Tick>().unwrap();
    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        set_progress.update(|p| {
            *p = if *p < 1.0 {
                (*p + 0.001f64).min(1.0)
            } else {
                0.0
            }
        });

        seq
    });

    view! {
        <Col v:constraint=constraint>
            <DemoGauge
                constraint=Constraint::Length(2)
                enhanced_graphics=enhanced_graphics
                progress=progress/>
            <DemoSparkline
                constraint=Constraint::Length(3)
                enhanced_graphics=enhanced_graphics/>
            <DemoLineGauge
                constraint=Constraint::Length(1)
                enhanced_graphics=enhanced_graphics progress=progress/>
        </Col>
    }
}

#[component]
fn DemoGauge(
    enhanced_graphics: bool,
    progress: ReadSignal<f64>,
    constraint: Constraint,
) -> impl Render {
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
) -> impl Render {
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
fn DemoSparkline(enhanced_graphics: bool, constraint: Constraint) -> impl Render {
    let mut rand_signal = RandomDistribution::new(0, 100);
    let sparkline_points = rand_signal.by_ref().take(300).collect();
    let sparkline_signal = RwSignal::new(RandomData {
        source: rand_signal,
        points: sparkline_points,
        tick_rate: 1,
    });

    let tick = use_context::<Tick>().unwrap();

    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        sparkline_signal.update(|s| s.on_tick());
        seq
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
fn Charts(enhanced_graphics: bool, constraint: Constraint) -> impl Render {
    let show_chart = RwSignal::new(true);

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Char('t') {
                show_chart.update(|s| *s = !*s);
            }
        }
    });

    view! {
        <Row v:constraint=constraint>
            <Col
                v:constraint=move || {
                    Constraint::Percentage(if show_chart.get() { 50 } else { 100 })
                }>
                <Row v:percentage=50>
                    <Col v:percentage=50>
                        <TaskList/>
                    </Col>
                    <Col v:percentage=50>
                        <Logs/>
                    </Col>
                </Row>
                <Row v:percentage=50>
                    <DemoBarChart enhanced_graphics=enhanced_graphics/>
                </Row>
            </Col>
            <Col
                v:constraint=move || {
                    Constraint::Percentage(if show_chart.get() { 50 } else { 0 })
                }>
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
fn TaskList() -> impl Render {
    let selected_task = RwSignal::<Option<usize>>::new(None);

    let update_current_task = move |change: i32| {
        selected_task.update(|sel| match sel {
            Some(s) => {
                let next = (*s as i32 + change).rem_euclid(TASKS.len() as i32);
                *sel = Some(next as usize);
            }
            None => *sel = Some(0),
        });
    };

    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                update_current_task(-1)
            } else if term_signal.code == KeyCode::Down {
                update_current_task(1);
            }
        }
    });

    view! {
        <StatefulList
            v:state= move || prop!(<ListState with_selected=selected_task.get()/>)
            block=prop!(<Block borders=Borders::ALL title="List"/>)
            highlight_style=prop!(<Style bold/>)
            highlight_symbol="> "
        > {
            TASKS
                .map(|t| {
                    prop! {
                        <ListItem>
                            <><Line><Span>{t}</Span></Line></>
                        </ListItem>
                    }
                })
            }
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
fn Logs() -> impl Render {
    let log_data = RwSignal::new(LOGS.to_vec());

    let tick = use_context::<Tick>().unwrap();
    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        log_data.update(|logs| {
            let log = logs.pop().unwrap();
            logs.insert(0, log);
        });
        seq
    });

    let logs = Memo::new(move |_| {
        let info_style = Style::default().fg(Color::Blue);
        let warning_style = Style::default().fg(Color::Yellow);
        let error_style = Style::default().fg(Color::Magenta);
        let critical_style = Style::default().fg(Color::Red);

        log_data
            .get()
            .into_iter()
            .map(|(evt, level)| {
                let style = match level {
                    "ERROR" => error_style,
                    "CRITICAL" => critical_style,
                    "WARNING" => warning_style,
                    _ => info_style,
                };
                (evt, level, style)
            })
            .collect::<Vec<_>>()
    });

    view! {
        <List
            block=prop!(<Block borders=Borders::ALL title="Logs"/>)
        > {
            logs.get().iter().map(|(evt, level, style)| {
                prop! {
                    <ListItem>
                        <Line>
                            <Span style=*style>{format!("{level:<9}")}</Span>
                            <Span>{*evt}</Span>
                        </Line>
                    </ListItem>
                }
            })
        }
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
fn DemoBarChart(enhanced_graphics: bool) -> impl Render {
    let bar_chart_data = RwSignal::new(EVENTS.to_vec());

    let tick = use_context::<Tick>().unwrap();

    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        bar_chart_data.update(|data| {
            let event = data.pop().unwrap();
            data.insert(0, event);
        });
        seq
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
fn DemoChart(enhanced_graphics: bool) -> impl Render {
    let mut sin1_data = SinData::new(0.2, 3.0, 18.0);
    let sin1 = RwSignal::new(RandomData::<SinData> {
        points: sin1_data.by_ref().take(100).collect(),
        source: sin1_data,
        tick_rate: 5,
    });

    let mut sin2_data = SinData::new(0.1, 2.0, 10.0);
    let sin2 = RwSignal::new(RandomData::<SinData> {
        points: sin2_data.by_ref().take(200).collect(),
        source: sin2_data,
        tick_rate: 10,
    });

    let window = RwSignal::new([0.0, 20.0]);
    let tick = use_context::<Tick>().unwrap();

    Effect::new(move |prev| {
        let seq = tick.0.get();
        if let Some(prev) = prev {
            if seq <= prev {
                return seq;
            }
        }
        window.update(|[start, end]| {
            *start += 1.0;
            *end += 1.0;
        });
        sin1.update(|s| s.on_tick());
        sin2.update(|s| s.on_tick());
        seq
    });

    let window_start = Memo::new(move |_| window.get()[0]);
    let window_end = Memo::new(move |_| window.get()[1]);

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
                            <Span>{
                                ((window_start.get() + window_end.get()) / 2.0).to_string()
                            }</Span>
                        },
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
fn Footer(constraint: Constraint) -> impl Render {
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
                "This is a paragraph with several lines.
                You can change style your text the way
                you want"             
            </Line>
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
fn DemoTable(servers: RwSignal<Vec<Server<'static>>>, constraint: Constraint) -> impl Render {
    let rows = Memo::new(move |_| {
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
) -> impl Render {
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
fn ColorsTable(constraint: Constraint) -> impl Render {
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

    view! {
        <Table
            v:constraint=constraint
            block=prop!(<Block title="Colors" borders=Borders::ALL/>)
        > {
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
            }
            {[
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3)
            ]}
        </Table>
    }
}
