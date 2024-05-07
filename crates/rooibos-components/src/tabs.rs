use reactive_graph::owner::StoredValue;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal;
use rooibos_dom::prelude::*;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, ComponentChildren)]
pub struct Tab {
    header: Line<'static>,
    value: String,
    #[children]
    #[builder(setter(transform = |p: impl IntoChildren| p.into_children()))]
    children: ChildrenFn,
}

#[component]
pub fn Tabs<S>(
    #[prop(children, into)] children: Vec<Tab>,
    #[prop(optional, into)] block: Option<Block<'static>>,
    padding: u16,
    highlight_style: Style,
    current_tab: S,
) -> impl Render
where
    S: Get<Value = String> + Send + Sync + 'static,
{
    let headers = StoredValue::new(
        children
            .iter()
            .map(|t| t.header.clone())
            .collect::<Vec<_>>(),
    );

    let cur_tab = Signal::derive(move || {
        let current_tab = current_tab.get();
        children.iter().enumerate().find_map(|(i, c)| {
            if c.value == current_tab {
                Some((c.children.clone(), i))
            } else {
                None
            }
        })
    });

    let block = StoredValue::new(block);

    view! {
        <col>
            <tabHeaders
                select=cur_tab.get().unwrap().1
                highlight_style=highlight_style
                v:length=padding * 2 + 1
                v:modify = {
                    |tab_headers: Tabs<'static>| {
                        if let Some(block) = block.get().unwrap() {
                            tab_headers.block(block)
                        } else {
                            tab_headers
                        }
                    }
                }
            >
                {headers.get().unwrap()}
            </tabHeaders>
            {move || cur_tab.get().unwrap().0()}
        </col>
    }
}
