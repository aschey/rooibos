use rooibos::dom::{col, constraint, ratio, row, widget_ref, Render};
use rooibos::tui::layout::Constraint::{self, *};
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::widgets::{Block, Cell, Row, Table};

pub(crate) fn tab2() -> impl Render {
    row![colors_table(Ratio(2, 1)), col![props(ratio(2, 1))]]
}

fn colors_table(table_constraint: Constraint) -> impl Render {
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

    widget_ref![
        props(constraint(table_constraint)),
        Table::new(
            colors.iter().map(|c| {
                Row::new(vec![
                    Cell::new(format!("{c:?}: ")),
                    Cell::new("Foreground".fg(*c)),
                    Cell::new("Background".bg(*c)),
                ])
            }),
            [Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)]
        )
        .block(Block::bordered().title("Colors"))
    ]
}
