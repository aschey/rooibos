use rooibos::reactive::{dom::Render, row, wgt};
use rooibos::tui::layout::Constraint;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::widgets::{Block, Cell, Row, Table};

pub(crate) fn tab2() -> impl Render {
    row![colors_table()]
}

fn colors_table() -> impl Render {
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

    wgt![
        Table::new(
            colors.iter().map(|c| {
                Row::new(vec![
                    Cell::new(format!("{c:?}: ")),
                    Cell::new("Foreground".fg(*c)),
                    Cell::new("Background".bg(*c)),
                ])
            }),
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3)
            ]
        )
        .block(Block::bordered().title("Colors"))
    ]
}
