use attribute_derive::Attribute;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Generics, Token, Visibility, WhereClause};

pub(crate) struct Model {
    name: Ident,
    make_builder: Ident,
    vis: Visibility,
    generics: Generics,
    stateful_render: Ident,
    pub(crate) stateful: bool,
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        let mut make_builder = Ident::new("_", Span::call_site());
        let mut stateful_render = Ident::new("_", Span::call_site());
        let mut vis = Visibility::Inherited;
        let mut generics = Generics::default();
        let mut where_clause: Option<WhereClause> = None;
        while !input.is_empty() {
            let _: syn::Token![,] = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match ident.to_string().as_str() {
                "visibility" => {
                    vis = input.parse()?;
                }
                "generics" => {
                    generics = input.parse()?;
                }
                "where_clause" => {
                    where_clause = Some(input.parse()?);
                }
                "make_builder" => {
                    make_builder = input.parse()?;
                }
                "stateful_render" => {
                    stateful_render = input.parse()?;
                }
                prop => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        format!("invalid property: {prop}"),
                    ));
                }
            }
        }
        generics.where_clause = where_clause;
        Ok(Model {
            name,
            vis,
            generics,
            make_builder,
            stateful_render,
            stateful: false,
        })
    }
}

impl ToTokens for Model {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(get_tokens(
            self.name.clone(),
            self.make_builder.clone(),
            self.vis.clone(),
            self.generics.clone(),
            self.stateful_render.clone(),
            self.stateful,
        ));
    }
}

#[derive(Clone, Debug, Attribute)]
#[attribute(ident = make_builder_trait)]
struct MakeBuilder {
    name: Ident,
}

#[derive(Clone, Debug, Attribute)]
#[attribute(ident = stateful_render_trait)]
struct StatefulRender {
    name: Ident,
}

pub(crate) fn derive_widget(input: DeriveInput) -> TokenStream {
    let make_builder = MakeBuilder::from_attributes(&input.attrs).unwrap();

    get_tokens(
        input.ident,
        make_builder.name,
        input.vis,
        input.generics,
        Ident::new("_", Span::call_site()),
        false,
    )
}

pub(crate) fn derive_stateful_widget(input: DeriveInput) -> TokenStream {
    let stateful_render = StatefulRender::from_attributes(&input.attrs).unwrap();

    get_tokens(
        input.ident,
        Ident::new("_", Span::call_site()),
        input.vis,
        input.generics,
        stateful_render.name,
        true,
    )
}

fn get_tokens(
    name: Ident,
    make_builder: Ident,
    vis: Visibility,
    generics: Generics,
    stateful_render: Ident,
    stateful: bool,
) -> TokenStream {
    let snake_name = if stateful {
        format!("Stateful{name}")
    } else {
        name.to_string()
    }
    .to_case(Case::Snake);

    let fn_name = Ident::new(&snake_name, Span::call_site());
    let props_name = if stateful {
        format!("Stateful{name}Props")
    } else {
        format!("{name}Props")
    };
    let props_name = Ident::new(&props_name, Span::call_site());

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut generics_static = generics.clone();
    if let Some(lifetime) = generics_static.lifetimes_mut().next() {
        lifetime.lifetime.ident = Ident::new("static", lifetime.lifetime.ident.span());
    }
    let (_, ty_generics_static, _) = generics_static.split_for_impl();

    if stateful {
        let state_name = Ident::new(&format!("{name}State"), Span::call_site());
        quote! {
            impl #impl_generics #stateful_render<#props_name #ty_generics>
            for ::std::cell::RefCell<#state_name> #where_clause
            {
                fn render_with_state(&mut self, widget: #props_name, frame: &mut Frame,
                    rect: Rect) {
                    frame.render_stateful_widget(widget, rect, &mut self.borrow_mut())
                }
            }

            impl #impl_generics #stateful_render<#props_name #ty_generics> for #state_name
            {
                fn render_with_state(&mut self, widget: #props_name, frame: &mut Frame,
                    rect: Rect) {
                    frame.render_stateful_widget(widget, rect, &mut self.clone())
                }
            }

            #vis type #props_name #ty_generics = #name #ty_generics;

            #vis fn #fn_name #impl_generics (
                _cx: Scope,
                props: #props_name #ty_generics_static,
                mut state: impl #stateful_render<#name #ty_generics> + 'static,
            ) -> impl View {
                move |frame: &mut Frame, rect: Rect| {
                    state.render_with_state(props.clone(), frame, rect);
                }
            }
        }
    } else {
        quote! {
            #vis type #props_name #ty_generics = #name #ty_generics;

            impl #impl_generics #make_builder for #props_name #ty_generics #where_clause {}

            #vis fn #fn_name #impl_generics (_cx: Scope, props: #props_name
                #ty_generics_static)
            -> impl View #where_clause {
                move |frame: &mut Frame, rect: Rect| frame.render_widget(props.clone(), rect)
            }
        }
    }
}
