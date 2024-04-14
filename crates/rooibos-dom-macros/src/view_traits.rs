use attribute_derive::FromAttr;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::ItemTrait;

#[derive(Clone, Debug, FromAttr)]
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
        }
    }
}
