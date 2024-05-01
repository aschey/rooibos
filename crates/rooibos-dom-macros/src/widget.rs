use attribute_derive::FromAttr;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Generics, LitBool, Token, Visibility, WhereClause};

#[derive(Clone)]
pub(crate) struct Model {
    name: Ident,
    name_override: Option<Ident>,
    make_builder: Ident,
    vis: Visibility,
    generics: Generics,
    state_generics: Generics,
    render_ref: bool,
    pub(crate) stateful: bool,
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        let mut make_builder = Ident::new("_", Span::call_site());
        let mut vis = Visibility::Inherited;
        let mut generics = Generics::default();
        let mut state_generics = Generics::default();
        let mut where_clause: Option<WhereClause> = None;
        let mut render_ref = true;
        let mut name_override = None;
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
                "state_generics" => {
                    state_generics = input.parse()?;
                }
                "where_clause" => {
                    where_clause = Some(input.parse()?);
                }
                "make_builder" => {
                    make_builder = input.parse()?;
                }
                "render_ref" => {
                    let render_bool: LitBool = input.parse()?;
                    render_ref = render_bool.value();
                }
                "name_override" => {
                    name_override = Some(input.parse()?);
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
            state_generics,
            make_builder,
            render_ref,
            stateful: false,
            name_override,
        })
    }
}

impl ToTokens for Model {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.get_tokens());
    }
}

impl Model {
    fn get_tokens(&self) -> TokenStream {
        let Model {
            name,
            name_override,
            make_builder,
            vis,
            generics,
            state_generics,
            render_ref,
            stateful,
        } = self.clone();
        let name_to_convert = name_override.unwrap_or_else(|| name.clone());
        let snake_name = if stateful {
            format!("Stateful{name_to_convert}")
        } else {
            name_to_convert.to_string()
        }
        .to_case(Case::Snake);

        let fn_name = Ident::new(&snake_name, Span::call_site());
        let props_name = if stateful {
            format!("Stateful{name_to_convert}Props")
        } else {
            format!("{name_to_convert}Props")
        };
        let props_name = Ident::new(&props_name, Span::call_site());

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let (_, state_ty_generics, _) = state_generics.split_for_impl();
        let mut generics_static = generics.clone();
        if let Some(lifetime) = generics_static.lifetimes_mut().next() {
            lifetime.lifetime.ident = Ident::new("static", lifetime.lifetime.ident.span());
        }
        let (_, ty_generics_static, _) = generics_static.split_for_impl();
        let type_name = Literal::string(&name.to_string());
        let render_props = if render_ref {
            quote!(&props)
        } else {
            quote!(props.clone())
        };

        if stateful {
            let state_name = Ident::new(&format!("{name}State"), Span::call_site());
            quote! {
                #vis type #props_name #ty_generics = #name #ty_generics;

                #vis fn #fn_name #impl_generics (
                    props: impl Fn() -> #props_name #ty_generics_static + 'static,
                    mut state: impl Fn() -> #state_name #state_ty_generics + 'static,
                ) -> DomWidget {
                    DomWidget::new(#type_name, move || {
                        let props = props();
                        let mut state = state();
                        move |frame: &mut Frame, rect: Rect| {
                            frame.render_stateful_widget(#render_props, rect, &mut state);
                        }
                    })
                }
            }
        } else {
            quote! {
                #vis type #props_name #ty_generics = #name #ty_generics;

                impl #impl_generics #make_builder for #props_name #ty_generics #where_clause {}

                #vis fn #fn_name #impl_generics (props: impl Fn() -> #props_name #ty_generics_static + 'static)
                -> DomWidget #where_clause {
                    DomWidget::new(#type_name, move || {
                        let props = props();
                        move |frame: &mut Frame, rect: Rect| {
                            frame.render_widget(#render_props, rect);
                        }

                    })
                }
            }
        }
    }
}

#[derive(Clone, Debug, FromAttr)]
#[attribute(ident = make_builder_trait)]
struct MakeBuilder {
    name: Ident,
}

#[derive(Clone, Debug, FromAttr)]
#[attribute(ident = render_ref)]
struct RenderRef(bool);

impl Default for RenderRef {
    fn default() -> Self {
        Self(true)
    }
}

pub(crate) fn derive_widget(input: DeriveInput) -> TokenStream {
    let make_builder = MakeBuilder::from_attributes(&input.attrs).unwrap();
    let render_ref = RenderRef::from_attributes(&input.attrs).unwrap_or_default();
    let model = Model {
        name: input.ident,
        make_builder: make_builder.name,
        vis: input.vis,
        generics: input.generics,
        state_generics: Generics::default(),
        stateful: false,
        render_ref: render_ref.0,
        name_override: None,
    };
    model.get_tokens()
}

pub(crate) fn derive_stateful_widget(input: DeriveInput) -> TokenStream {
    let render_ref = RenderRef::from_attributes(&input.attrs).unwrap_or_default();
    // get_tokens(
    //     input.ident,
    //     Ident::new("_", Span::call_site()),
    //     input.vis,
    //     input.generics,
    //     Generics::default(),
    //     true,
    //     render_ref.0,
    // )

    let model = Model {
        name: input.ident,
        make_builder: Ident::new("_", Span::call_site()),
        vis: input.vis,
        generics: input.generics,
        state_generics: Generics::default(),
        stateful: true,
        render_ref: render_ref.0,
        name_override: None,
    };
    model.get_tokens()
}
