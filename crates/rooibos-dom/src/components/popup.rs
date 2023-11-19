use crate::prelude::*;

#[component]
pub fn Popup<V>(#[prop(children)] children: V, percent_x: u16, percent_y: u16) -> impl IntoView
where
    V: IntoDomNode,
{
    let inverse_y = (100 - percent_y) / 2;
    let inverse_x = (100 - percent_x) / 2;

    view! {
        <Column>
            <Row v:percentage=inverse_y />
            <Row v:percentage=percent_y>
                <Column v:percentage=inverse_x />
                <Column v:percentage=percent_x>
                    <Overlay>
                        <Clear/>
                        {children}
                    </Overlay>
                </Column>
                <Column v:percentage=inverse_x />
            </Row>
            <Row v:percentage=inverse_y />
        </Column>
    }
}
