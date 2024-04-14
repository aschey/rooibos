use rooibos::prelude::*;

#[component]
pub(crate) fn Tab2() -> impl Render {
    view! {
        <Row>
            <ColorsTable constraint=Constraint::Ratio(2, 1)/>
            <Col v:ratio=(2,1)/>
        </Row>
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
