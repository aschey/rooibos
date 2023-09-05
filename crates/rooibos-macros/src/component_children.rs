use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, ToTokens};
use regex::Regex;
use syn::{Data, DeriveInput, Fields, Meta};

macro_rules! regex {
    ($name: ident, $re: literal $(,) ?) => {
        static $name: Lazy<Regex> = Lazy::new(|| Regex::new($re).expect("Regex failed to compile"));
    };
}

regex!(
    TRANSFORM_RE,
    r"transform\s*=\s*\|\s*[a-zA-Z_]+\s*:\s*(.*)\s*\|"
);
regex!(INTO_RE, r"setter\s*\(.*\s*into,?\s*\)");

pub(crate) fn parse(input: DeriveInput) -> TokenStream {
    let Data::Struct(derive_struct) = &input.data else {
        abort_call_site!("ComponentChildren can only be used on structs");
    };
    let Fields::Named(fields) = &derive_struct.fields else {
        abort_call_site!("ComponentChildren can only be used on structs with named fields");
    };

    let find_children: Vec<_> = fields
        .named
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            f.attrs.iter().any(|a| {
                if let Meta::Path(path) = &a.meta {
                    return path.is_ident("children");
                }
                false
            })
        })
        .collect();

    if find_children.is_empty() {
        return Default::default();
    } else if find_children.len() > 1 {
        abort!(fields, "Only one field can have the `children` attribute");
    }

    let (children_index, children_field) = find_children.first().expect("length check");
    let fn_param = children_field.attrs.iter().find_map(|a| {
        if let Meta::List(meta_list) = &a.meta {
            if meta_list.path.segments.first().unwrap().ident == "builder" {
                let attr_str = a.to_token_stream().to_string().replace('\n', "");

                if let Some(captures) = TRANSFORM_RE.captures(&attr_str) {
                    if let Some(func) = captures.get(1) {
                        return Some(syn::parse_str::<syn::Type>(func.as_str()).unwrap());
                    }
                }

                if INTO_RE.is_match(&attr_str) {
                    let return_type = children_field.ty.clone();
                    return Some(syn::parse_quote!(impl Into<#return_type>));
                }
            }
        }
        None
    });

    let props_len = fields.named.len();
    let children_prop = children_field.ident.clone().unwrap();
    let fn_param_type = fn_param
        .map(|f| f.to_token_stream())
        .unwrap_or_else(|| children_field.ty.clone().to_token_stream());
    let prop_type = children_field.ty.clone();
    let generics_tokens: Vec<_> = input
        .generics
        .type_params()
        .map(|p| p.ident.to_token_stream())
        .collect();

    let return_generics = if generics_tokens.is_empty() {
        quote! {}
    } else {
        quote! { #(#generics_tokens),*, }
    };
    let before_args: Vec<TokenStream> = (0..*children_index).map(|_| quote! {()}).collect();
    // -1 for current arg
    let extra_args: Vec<TokenStream> = (0..props_len - children_index - 1)
        .map(|_| quote! {()})
        .collect();

    let args = [before_args, vec![quote!((#prop_type,))], extra_args].concat();
    let (impl_generics, generics, where_clause) = input.generics.split_for_impl();
    let props_name = input.ident;
    let props_builder_name = format_ident!("{props_name}Builder");

    quote! {
        impl #impl_generics #props_name #generics #where_clause {
            pub fn new(#children_prop: #fn_param_type) -> #props_builder_name <#return_generics (#(#args),*)> {
                Self::builder().#children_prop(#children_prop)
            }
        }
    }
}
