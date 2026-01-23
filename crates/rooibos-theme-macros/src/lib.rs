use itertools::Itertools;
use manyhow::{Emitter, bail, manyhow};
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Path, Type};

fn is_style(s: &str) -> bool {
    matches!(
        s,
        "Style" | "rooibos_theme::Style" | "ratatui::style::Style" | "ratatui_core::style::Style"
    )
}

fn is_color(s: &str) -> bool {
    matches!(
        s,
        "Color" | "rooibos_theme::Color" | "ratatui::style::Color" | "ratatui_core::style::Color"
    )
}

fn filter_fields<F>(fields: &Fields, filter: F) -> Vec<&Ident>
where
    F: Fn(&str) -> bool,
{
    fields
        .iter()
        .filter_map(|f| {
            let ty = f.ty.to_token_stream().to_string().replace(" ", "");
            if filter(ty.as_str()) {
                f.ident.as_ref()
            } else {
                None
            }
        })
        .collect()
}

fn get_style_fields(fields: &Fields) -> Vec<&Ident> {
    filter_fields(fields, is_style)
}

fn get_color_fields(fields: &Fields) -> Vec<&Ident> {
    filter_fields(fields, is_color)
}

fn get_other_fields(fields: &Fields) -> Vec<(&Ident, &Path)> {
    fields
        .iter()
        .filter_map(|f| {
            let ty = f.ty.to_token_stream().to_string().replace(" ", "");
            if !f.attrs.iter().any(|a| a.meta.path().is_ident("subtheme"))
                && !is_color(&ty)
                && !is_style(&ty)
                && let Type::Path(path) = &f.ty
                && path.qself.is_none()
            {
                f.ident.as_ref().map(|i| (i, &path.path))
            } else {
                None
            }
        })
        .collect()
}

fn get_subtheme_fields(fields: &Fields) -> Vec<(&Ident, &Type)> {
    fields
        .iter()
        .filter_map(|f| {
            if f.attrs.iter().any(|a| a.meta.path().is_ident("subtheme")) {
                f.ident.as_ref().map(|i| (i, &f.ty))
            } else {
                None
            }
        })
        .collect()
}

#[manyhow(proc_macro_derive(Theme, attributes(subtheme, theme)))]
pub fn derive_theme(input: DeriveInput, _emitter: &mut Emitter) -> manyhow::Result {
    let data_struct = if let Data::Struct(data_struct) = &input.data {
        Some(data_struct)
    } else {
        None
    };
    if input.generics.lifetimes().next().is_some() || input.generics.type_params().next().is_some()
    {
        bail!(
            input.span(),
            "Lifetimes and generics are not supported here"
        );
    }
    let struct_name = &input.ident;
    let struct_name_upper = struct_name.to_string().to_ascii_uppercase();

    let _global_theme = Ident::new(
        &format!("__{struct_name_upper}__GLOBAL_THEME"),
        Span::call_site(),
    );
    let _local_theme = Ident::new(
        &format!("__{struct_name_upper}__LOCAL_THEME"),
        Span::call_site(),
    );

    let rooibos_theme = get_import("rooibos-theme").expect("rooibos-theme not found");
    let subtheme_fields = data_struct
        .map(|d| get_subtheme_fields(&d.fields))
        .unwrap_or_default();
    let other_fields = data_struct
        .map(|d| get_other_fields(&d.fields))
        .unwrap_or_default();

    let subtheme_set: TokenStream = subtheme_fields
        .iter()
        .map(|(f, _)| quote!(#rooibos_theme::SetTheme::set(&self.#f);))
        .collect();

    let style_trait = Ident::new(&(struct_name.to_string() + "Style"), Span::call_site());
    let style_ext_trait = Ident::new(&(struct_name.to_string() + "StyleExt"), Span::call_site());
    let color_trait = Ident::new(&(struct_name.to_string() + "ColorTheme"), Span::call_site());
    let color_ext_trait = Ident::new(
        &(struct_name.to_string() + "ColorThemeExt"),
        Span::call_site(),
    );
    let signal = quote!(rooibos_reactive::graph::wrappers::read::Signal);

    let other_traits: TokenStream = other_fields
        .iter()
        .chunk_by(|(_, ty)| ty)
        .into_iter()
        .map(|(ty, group)| {
            let trait_name = Ident::new(
                &(struct_name.to_string()
                    + &ty
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<String>()
                    + "Ext"),
                Span::call_site(),
            );
            let methods: Vec<_> = group
                .into_iter()
                .map(|(ident, _)| {
                    (
                        quote! {
                            fn #ident() -> #signal<#ty>;
                        },
                        quote! {
                            fn #ident() -> #signal<#ty> {
                                use rooibos_reactive::IntoSignal;
                                use rooibos_reactive::graph::traits::Track;

                                let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                                (move || {
                                    trigger.track();
                                    rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| t.#ident.clone()).unwrap()
                                }).signal()
                            }
                        },
                    )
                })
                .collect();

            let method_defs: TokenStream = methods.iter().map(|(m, _)| m.clone()).collect();
            let method_impls: TokenStream = methods.into_iter().map(|(_, m)| m).collect();
            quote! {
                pub trait #trait_name {
                    #method_defs
                }

                impl #trait_name for #ty {
                    #method_impls
                }
            }
        })
        .collect();

    let style_fields = data_struct
        .map(|d| get_style_fields(&d.fields))
        .unwrap_or_default();
    let color_fields = data_struct
        .map(|d| get_color_fields(&d.fields))
        .unwrap_or_default();

    let style_trait_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&format!("style_{f}"), Span::call_site());
            quote! {
                fn #style_fn(self) -> #signal<T>;
            }
        })
        .collect();

    let ratatui = get_import("ratatui")
        .or_else(|| get_import("ratatui-core"))
        .expect("ratatui not found");

    let style_impl_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&format!("style_{f}"), Span::call_site());
            quote! {
                fn #style_fn(self) -> #signal<T> {
                    use rooibos_reactive::IntoSignal;
                    use rooibos_reactive::graph::traits::Track;

                    let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    let this = self.clone();
                    (move || {
                        trigger.track();
                        rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| this.clone().set_style(t.#f)).unwrap()
                    }).signal()
                }
            }
        })
        .collect();

    let color_trait_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let fg_fn = Ident::new(&format!("fg_{f}"), Span::call_site());
            let bg_fn = Ident::new(&format!("bg_{f}"), Span::call_site());
            let underline_fn = Ident::new(&format!("underline_{f}"), Span::call_site());
            quote! {
                fn #fg_fn(self) -> #signal<T>;
                fn #bg_fn(self) -> #signal<T>;
                fn #underline_fn(self) -> #signal<T>;
            }
        })
        .collect();

    let color_ext_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let color_fn = Ident::new(&f.to_string(), Span::call_site());

            quote! {
                fn #color_fn() -> #signal<Self>;
            }
        })
        .collect();

    let style_ext_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&f.to_string(), Span::call_site());

            quote! {
                fn #style_fn() -> #signal<Self>;
            }
        })
        .collect();

    let impl_fns: TokenStream = data_struct
        .map(|d| {
            d.fields
                .iter()
                .filter_map(|f| {
                    if let Some(ident) = &f.ident {
                        let ty = &f.ty;
                        Some(quote! {
                            pub fn #ident() -> #signal<#ty> {
                                use rooibos_reactive::IntoSignal;
                                use rooibos_reactive::graph::traits::Track;

                                let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                                (move || {
                                    trigger.track();
                                    rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| t.#ident.clone()).unwrap()
                                }).signal()
                            }
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let color_impl_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let fg_fn = Ident::new(&format!("fg_{f}"), Span::call_site());
            let bg_fn = Ident::new(&format!("bg_{f}"), Span::call_site());
            let underline_fn = Ident::new(&format!("underline_{f}"), Span::call_site());

            quote! {
                fn #fg_fn(self) -> #signal<T> {
                    use rooibos_reactive::IntoSignal;
                    use rooibos_reactive::graph::traits::Track;

                    let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    let this = self.clone();
                    (move || {
                        trigger.track();
                        rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| this.clone().fg(t.#f)).unwrap()
                    }).signal()
                }

                fn #bg_fn(self) -> #signal<T> {
                    use rooibos_reactive::IntoSignal;
                    use rooibos_reactive::graph::traits::Track;

                    let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    let this = self.clone();
                    (move || {
                        trigger.track();
                        rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| this.clone().bg(t.#f)).unwrap()
                    }).signal()
                }

                fn #underline_fn(self) -> #signal<T> {
                    use rooibos_reactive::IntoSignal;
                    use rooibos_reactive::graph::traits::Track;

                    let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    let this = self.clone();
                    (move || {
                        trigger.track();
                        rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| this.clone().underline_color(t.#f)).unwrap()
                    }).signal()
                }

            }
        })
        .collect();

    let color_ext_impl_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let color_fn = Ident::new(&f.to_string(), Span::call_site());

            quote! {
                fn #color_fn() -> #signal<Self> {
                    use rooibos_reactive::IntoSignal;
                    use rooibos_reactive::graph::traits::Track;

                    let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    (move || {
                        trigger.track();
                        rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| t.#f.into()).unwrap()
                    }).signal()
                }
            }
        })
        .collect();

    let style_ext_impl_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&f.to_string(), Span::call_site());

            quote! {
                fn #style_fn() -> #signal<Self> {
                    use rooibos_reactive::IntoSignal;
                    use rooibos_reactive::graph::traits::Track;

                    let trigger = rooibos_reactive::graph::owner::with_context::<rooibos_theme::ThemeContext, _>(|t| t.trigger.clone()).unwrap();
                    (move || {
                        trigger.track();
                        rooibos_reactive::graph::owner::with_context::<#struct_name, _>(|t| t.#f.into()).unwrap()
                    }).signal()
                }
            }
        })
        .collect();

    Ok(quote! {
        impl #rooibos_theme::SetTheme for #struct_name {
            type Theme = Self;

            fn set(&self) {
                #subtheme_set
                if rooibos_reactive::graph::owner::update_context::<#struct_name, _>(|val| {
                    std::mem::replace(val, self.clone())
                }).is_none() {
                    rooibos_reactive::graph::owner::provide_context(self.clone());
                }
            }

            fn current() -> Self {
                rooibos_reactive::graph::owner::use_context::<#struct_name>().unwrap()
            }

            fn with_theme< F, T>(f: F) -> T
            where
                F: FnOnce(&Self::Theme) -> T
            {
                rooibos_reactive::graph::owner::with_context::<#struct_name, _>(f).unwrap()
            }
        }

        impl #struct_name {
            #impl_fns
        }

        pub trait #style_trait<T> where T: Send + Sync + 'static {
            #style_trait_fns
        }

        impl<T, U> #style_trait<T> for U
        where
            T: Send + Sync + 'static,
            U: #rooibos_theme::Styled<Item = T> + Clone + Send + Sync + 'static
        {
            #style_impl_fns
        }

        pub trait #color_trait<T>
        where
            T: Send + Sync + 'static,
            Self: Clone + Send + Sync + 'static
        {
            #color_trait_fns
        }

        impl<'a, T, U> #color_trait<T> for U
        where
            T: Send + Sync + 'static,
            U: #rooibos_theme::Stylize<'a, T> + Clone + Send + Sync + 'static,
        {
            #color_impl_fns
        }

        pub trait #color_ext_trait where Self: Sized + Send + Sync + 'static {
            #color_ext_fns
        }

        impl #color_ext_trait for #rooibos_theme::Color {
            #color_ext_impl_fns
        }

        impl #color_ext_trait for #ratatui::style::Color {
            #color_ext_impl_fns
        }

        pub trait #style_ext_trait where Self: Sized + Send + Sync + 'static {
            #style_ext_fns
        }

        impl #style_ext_trait for #rooibos_theme::Style {
            #style_ext_impl_fns
        }

        impl #style_ext_trait for #ratatui::style::Style {
            #style_ext_impl_fns
        }

        #other_traits
    })
}

fn get_import(crate_name_str: &str) -> Option<TokenStream> {
    let found_crate = crate_name(crate_name_str).ok()?;
    match found_crate {
        FoundCrate::Itself => {
            let ident = Ident::new(&crate_name_str.replace("-", "_"), Span::call_site());
            Some(quote!(#ident))
        }
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            Some(quote!(#ident))
        }
    }
}
