use rooibos::reactive::graph::computed::Memo;
use rooibos::reactive::graph::owner::StoredValue;
use rooibos::reactive::graph::traits::{Get, GetValue};
use rooibos::reactive::{dom::Render, col, row, wgt, width};
use rooibos::tui::layout::Constraint;
use rooibos::tui::style::{Color, Style, Stylize};
use rooibos::tui::symbols;
use rooibos::tui::widgets::canvas::{self, Canvas, Circle, Context, Map, MapResolution, Rectangle};
use rooibos::tui::widgets::{Block, Row, Table};

pub(crate) fn tab1() -> impl Render {
    let servers = StoredValue::new(vec![
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

    row![col![props(width!(30.%)), demo_table(servers)], col![
        props(width!(70.%)),
        demo_map(servers, true)
    ]]
}

#[derive(Clone)]
pub struct Server<'a> {
    pub name: &'a str,
    pub location: &'a str,
    pub coords: (f64, f64),
    pub status: &'a str,
}

fn demo_table(servers: StoredValue<Vec<Server<'static>>>) -> impl Render {
    let rows = Memo::new(move |_| {
        servers
            .get_value()
            .into_iter()
            .map(|s| {
                let style = if s.status == "Up" {
                    Style::new().green()
                } else {
                    Style::new().red().rapid_blink().crossed_out()
                };
                Row::new(vec![s.name, s.location, s.status]).style(style)
            })
            .collect::<Vec<_>>()
    });

    wgt!(
        Table::new(rows.get(), [
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ])
        .header(
            Row::new(vec!["Server", "Location", "Status"])
                .yellow()
                .bottom_margin(1)
        )
        .block(Block::bordered().title("Servers"))
    )
}

fn demo_map(servers: StoredValue<Vec<Server<'static>>>, enhanced_graphics: bool) -> impl Render {
    let paint_map = move |ctx: &mut Context<'_>| {
        let servers = servers.get_value();
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
            ctx.print(server.coords.1, server.coords.0, "X".fg(color));
        }
    };

    wgt!(
        Canvas::default()
            .block(Block::bordered().title("World"))
            .paint(paint_map)
            .marker(if enhanced_graphics {
                symbols::Marker::Braille
            } else {
                symbols::Marker::Dot
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
    )
}
