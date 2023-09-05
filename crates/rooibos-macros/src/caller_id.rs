use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::ToTokens;
use syn::{Data, DeriveInput, Fields};

pub(crate) fn parse(mut input: DeriveInput) -> TokenStream {
    let Data::Struct(derive_struct) = &mut input.data else {
        abort_call_site!("caller_id can only be used on structs");
    };
    let Fields::Named(fields) = &mut derive_struct.fields else {
        abort_call_site!("caller_id can only be used on structs with named fields");
    };

    fields.named.push(syn::Field {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        mutability: syn::FieldMutability::None,
        ident: Some(syn::parse_quote!(__caller_id)),
        ty: syn::parse_quote!(u32),
        colon_token: Some(syn::parse_quote!(:)),
    });

    input.into_token_stream()
}
