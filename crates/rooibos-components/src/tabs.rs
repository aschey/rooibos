use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::{MaybeProp, MaybeSignal, Signal};
use rooibos_dom::prelude::*;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, ComponentChildren, Clone)]
pub struct Tab {
    #[builder(setter(into))]
    header: MaybeSignal<Line<'static>>,
    #[builder(setter(into))]
    value: MaybeSignal<String>,
    #[children]
    #[builder(setter(transform = |p: impl IntoChildren| p.into_children()))]
    children: ChildrenFn,
}

#[component]
pub fn Tabs<S>(
    #[prop(children, into)] children: MaybeSignal<Vec<Tab>>,
    #[prop(optional, into)] block: MaybeProp<Block<'static>>,
    #[prop(optional, into)] padding: MaybeProp<u16>,
    highlight_style: Style,
    current_tab: S,
) -> impl Render
where
    S: Get<Value = String> + Send + Sync + 'static,
{
    let children_ = children.clone();
    let headers = Signal::derive(move || {
        children_
            .get()
            .iter()
            .map(|t| t.header.get())
            .collect::<Vec<_>>()
    });

    let cur_tab = Signal::derive(move || {
        let current_tab = current_tab.get();
        children.get().iter().enumerate().find_map(|(i, c)| {
            if c.value.get() == current_tab {
                Some((c.children.clone(), i))
            } else {
                None
            }
        })
    });

    view! {
        <col>
            <tabHeaders
                select=cur_tab.get().unwrap().1
                highlight_style=highlight_style
                v:length=padding.get().unwrap_or(0) * 2 + 1
                v:modify = {
                    |tab_headers: Tabs<'static>| {
                        if let Some(block) = block.get() {
                            tab_headers.block(block)
                        } else {
                            tab_headers
                        }
                    }
                }
            >
                {headers.get()}
            </tabHeaders>
            {move || cur_tab.get().unwrap().0()}
        </col>
    }
}
