use convert_case::{Case, Casing};
use manyhow::{bail, Emitter, ErrorMessage};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_str, Data, DataEnum, DataStruct, DeriveInput, Expr, Fields, Generics, Ident, Type,
};

use crate::get_components_import;

struct StructField {
    ident: Ident,
    optional: bool,
    components: TokenStream,
}

impl StructField {
    fn const_name(&self) -> TokenStream {
        let name_str = self.ident.to_string().to_ascii_lowercase();
        let name_upper = Ident::new(&name_str.to_ascii_uppercase(), Span::call_site());
        let components = &self.components;
        if self.optional {
            quote!(const #name_upper: #components::Query = #components::Query(::std::borrow::Cow::Borrowed(#name_str));)
        } else {
            quote!(const #name_upper: #components::Param = #components::Param(::std::borrow::Cow::Borrowed(#name_str));)
        }
    }

    fn add_to_route(&self) -> TokenStream {
        let ident = &self.ident;
        let ident_assign = format!("{ident}=");
        if self.optional {
            quote! {
                if let Some(#ident) = &self.#ident {
                    if base_added {
                        url += "&";
                    } else {
                        url += "?";
                        base_added = true;
                    }
                    url += #ident_assign;
                    url += &#ident.to_string();
                }

            }
        } else {
            quote! {
                url += "/";
                url += &self.#ident.to_string();
            }
        }
    }

    fn add_to_route_template(&self) -> String {
        let ident = &self.ident;
        if self.optional {
            "".to_string()
        } else {
            format!("{{{ident}}}")
        }
    }
}

pub(crate) fn generate(input: DeriveInput, emitter: &mut Emitter) -> manyhow::Result {
    let components = get_components_import();
    match input.data {
        Data::Struct(struct_input) => generate_struct(
            input.ident,
            input.generics,
            struct_input,
            components,
            emitter,
        ),
        Data::Enum(enum_input) => {
            generate_enum(input.ident, input.generics, enum_input, components, emitter)
        }
        Data::Union(_) => bail!("Route cannot be derived on this type"),
    }
}

fn generate_struct(
    ident: Ident,
    generics: Generics,
    struct_input: DataStruct,
    components: TokenStream,
    emitter: &mut Emitter,
) -> manyhow::Result {
    let fields = get_struct_fields(struct_input, components.clone(), emitter);

    let base_name = ident.to_string().to_case(Case::Kebab);
    let base_route = format!("/{base_name}");

    let consts: Vec<_> = fields.iter().map(|f| f.const_name()).collect();
    let to_route: Vec<_> = fields.iter().map(|f| f.add_to_route()).collect();
    let extra_route_paths = fields
        .iter()
        .map(|f| f.add_to_route_template())
        .collect::<Vec<_>>()
        .join("/");
    let mut route_template = base_route.clone();
    if !extra_route_paths.is_empty() {
        route_template += "/";
    }
    route_template += &extra_route_paths;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#consts)*
        }

        impl #impl_generics #components::ToRoute for #ident #ty_generics #where_clause {
            fn to_route(&self) -> String {
                let mut url = #base_route.to_string();
                let mut base_added = false;
                #(#to_route)*
                url
            }
        }

        impl #impl_generics #components::ToRouteTemplateStatic for #ident #ty_generics #where_clause {
            fn to_route_template() -> &'static str {
                #route_template
            }
        }
    })
}

fn get_struct_fields(
    struct_input: DataStruct,
    components: TokenStream,
    emitter: &mut Emitter,
) -> Vec<StructField> {
    struct_input
        .fields
        .iter()
        .filter_map(|f| {
            let Some(ident) = &f.ident else {
                emitter.emit(ErrorMessage::spanned(
                    &f.ident,
                    "Route cannot be derived on unnamed fields",
                ));
                return None;
            };
            if let Type::Path(path) = &f.ty {
                let idents_of_path = path.path.segments.iter().fold(String::new(), |mut acc, v| {
                    acc.push_str(&v.ident.to_string());
                    acc.push(':');
                    acc
                });

                let optional = vec!["Option:", "std:option:Option:", "core:option:Option:"]
                    .into_iter()
                    .any(|s| idents_of_path == s);
                Some(StructField {
                    ident: ident.clone(),
                    optional,
                    components: components.clone(),
                })
            } else {
                Some(StructField {
                    ident: ident.clone(),
                    optional: false,
                    components: components.clone(),
                })
            }
        })
        .collect()
}

fn generate_enum(
    ident: Ident,
    generics: Generics,
    enum_input: DataEnum,
    components: TokenStream,
    emitter: &mut Emitter,
) -> manyhow::Result {
    let fields = get_enum_fields(enum_input, emitter);
    let routes = fields.iter().map(|f| f.field_to_route());
    let route_templates = fields.iter().map(|f| f.field_to_route_template());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #components::ToRoute for #ident #ty_generics #where_clause {
            fn to_route(&self) -> String {
                match self {
                    #(#routes),*
                }
            }
        }

        impl #impl_generics #components::ToRouteTemplate for #ident #ty_generics #where_clause {
            fn to_route_template(&self) -> &'static str {
                match self {
                    #(#route_templates),*
                }
            }
        }
    })
}

struct EnumField {
    ident: Ident,
}

impl EnumField {
    fn field_to_route(&self) -> TokenStream {
        let variant: Expr = parse_str(&format!("Self::{}", self.ident)).unwrap();
        let route = "/".to_string() + &(self.ident.to_string().to_case(Case::Kebab));
        quote!(#variant => #route.to_string())
    }

    fn field_to_route_template(&self) -> TokenStream {
        let variant: Expr = parse_str(&format!("Self::{}", self.ident)).unwrap();
        let route = "/".to_string() + &(self.ident.to_string().to_case(Case::Kebab));
        quote!(#variant => #route)
    }
}

fn get_enum_fields(enum_input: DataEnum, emitter: &mut Emitter) -> Vec<EnumField> {
    enum_input
        .variants
        .iter()
        .filter_map(|v| {
            if v.fields != Fields::Unit {
                emitter.emit(ErrorMessage::spanned(
                    &v.ident,
                    "Route cannot be derived on enum variants with fields",
                ));
                return None;
            }
            Some(EnumField {
                ident: v.ident.clone(),
            })
        })
        .collect()
}
