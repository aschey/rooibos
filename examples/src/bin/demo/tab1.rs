use rooibos::prelude::canvas::{Circle, Context, Map, MapResolution, Rectangle};
use rooibos::prelude::Constraint::*;
use rooibos::prelude::*;
use rooibos::reactive::computed::Memo;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::Get;

#[component]
pub(crate) fn Tab1() -> impl Render {
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
        <row>
            <DemoTable
                constraint=Percentage(30)
                servers=servers/>
            <DemoMap
                constraint=Percentage(70)
                enhanced_graphics=true
                servers=servers/>
        </row>
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
        <table
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
                Length(15),
                Length(15),
                Length(10),
            ]}
        </table>
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
        <canvas
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
