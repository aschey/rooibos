use crate::prelude::*;

#[component]
pub fn Popup<IV>(
    #[prop(children)] children: IV,
    percent_x: u16,
    percent_y: u16,
    #[prop(default=None)] constraint: Option<Constraint>,
) -> impl IntoView
where
    IV: IntoView,
{
    let inverse_y = (100 - percent_y) / 2;
    let inverse_x = (100 - percent_x) / 2;

    view! {
        <Col v:constraint=constraint.unwrap_or_default()>
            <Row v:percentage=inverse_y />
            <Row v:percentage=percent_y>
                <Col v:percentage=inverse_x />
                <Col v:percentage=percent_x>
                    <Overlay>
                        <Clear/>
                        {children}
                    </Overlay>
                </Col>
                <Col v:percentage=inverse_x />
            </Row>
            <Row v:percentage=inverse_y />
        </Col>
    }
}
