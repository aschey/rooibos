use attribute_derive::Attribute;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::Parse;
use syn::{ItemTrait, Token, Visibility};

#[derive(Clone, Debug, Attribute)]
#[attribute(ident = make_builder)]
struct MakeBuilder {
    #[attribute(optional)]
    suffix: String,
}

pub(crate) fn attr_make_builder(attr: TokenStream, input: TokenStream) -> TokenStream {
    let trait_def: ItemTrait = syn::parse2(input).unwrap();
    let vis = trait_def.vis.clone();
    let trait_name = trait_def.ident.clone();
    let make_builder: MakeBuilder = syn::parse2(attr).unwrap();
    let builder_facade = Ident::new(
        &("BuilderFacade".to_string() + &make_builder.suffix),
        Span::call_site(),
    );
    let build_facade = Ident::new(
        &("BuildFacade".to_string() + &make_builder.suffix),
        Span::call_site(),
    );
    quote! {
        #trait_def

        #vis trait #builder_facade {
            fn builder() -> Self;
        }

        #vis trait #build_facade {
            fn build(self) -> Self;
            fn __caller_id(self, caller_id: u64) -> Self;
        }


        impl<T> #builder_facade for T
        where
            T: #trait_name + Default,
        {
            fn builder() -> Self {
                Self::default()
            }
        }

        impl<T> #build_facade for T
        where
            T: #trait_name,
        {
            fn build(self) -> Self {
                self
            }

            fn __caller_id(self, _caller_id: u64) -> Self {
                self
            }
        }
    }
}

pub(crate) struct StatefulModel {
    vis: Visibility,
    name: Ident,
}

impl Parse for StatefulModel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let mut vis = Visibility::Inherited;
        if !input.is_empty() {
            let _: syn::Token![,] = input.parse()?;
            let ident: syn::Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match ident.to_string().as_str() {
                "visibility" => {
                    vis = input.parse()?;
                }
                prop => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        format!("invalid property: {prop}"),
                    ));
                }
            }
        }

        Ok(Self { vis, name })
    }
}

impl ToTokens for StatefulModel {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.vis;
        let name = &self.name;
        let res = quote! {
            #vis trait #name<B, W>
            where
                B: Backend,
                W: StatefulWidget,
            {
                fn render_with_state(&mut self, widget: W, frame: &mut Frame<B>, rect: Rect);
            }
        };

        tokens.append_all(res);
    }
}
