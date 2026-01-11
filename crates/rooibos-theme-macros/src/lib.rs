use darling::FromDeriveInput;
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

fn method_name(ident: Ident, opts: &ThemeOpts) -> Ident {
    if let Some(prefix) = &opts.prefix {
        Ident::new(&format!("{prefix}_{ident}"), Span::call_site())
    } else {
        ident
    }
}

#[derive(Debug, FromDeriveInput, Default)]
#[darling(attributes(theme))]
struct ThemeOpts {
    prefix: Option<String>,
}

#[manyhow(proc_macro_derive(Theme, attributes(subtheme, theme)))]
pub fn derive_theme(input: DeriveInput, emitter: &mut Emitter) -> manyhow::Result {
    let opts = ThemeOpts::from_derive_input(&input)
        .inspect_err(|e| {
            emitter.emit(manyhow::ErrorMessage::new(e.span(), e.to_string()));
        })
        .unwrap_or_default();
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

    let global_theme = Ident::new(
        &format!("__{struct_name_upper}__GLOBAL_THEME"),
        Span::call_site(),
    );
    let local_theme = Ident::new(
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

    let subtheme_set_local: TokenStream = subtheme_fields
        .iter()
        .map(|(f, _)| quote!(#rooibos_theme::SetTheme::set_local(&self.#f);))
        .collect();
    let subtheme_set_global: TokenStream = subtheme_fields
        .iter()
        .map(|(f, _)| quote!(#rooibos_theme::SetTheme::set_global(&self.#f);))
        .collect();
    let subtheme_unset_local: TokenStream = subtheme_fields
        .iter()
        .map(|(_, ty)| quote!(<#ty as #rooibos_theme::SetTheme>::unset_local();))
        .collect();

    let style_trait = Ident::new(&(struct_name.to_string() + "Style"), Span::call_site());
    let style_ext_trait = Ident::new(&(struct_name.to_string() + "StyleExt"), Span::call_site());
    let color_trait = Ident::new(&(struct_name.to_string() + "ColorTheme"), Span::call_site());
    let color_ext_trait = Ident::new(
        &(struct_name.to_string() + "ColorThemeExt"),
        Span::call_site(),
    );

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
                    let method = method_name((*ident).clone(), &opts);
                    (
                        quote! {
                            fn #method() -> #ty;
                        },
                        quote! {
                            fn #method() -> #ty {
                                use #rooibos_theme::SetTheme;
                                #struct_name::with_theme(|t| t.#ident.clone())
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
                fn #style_fn(self) -> T;
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
            let style_fn = method_name(style_fn, &opts);
            quote! {
                fn #style_fn(self) -> T {
                    <#struct_name as #rooibos_theme::SetTheme>::with_theme(|t| self.set_style(t.#f))
                }
            }
        })
        .collect();

    let color_trait_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let fg_fn = Ident::new(&format!("fg_{f}"), Span::call_site());
            let fg_fn = method_name(fg_fn, &opts);
            let bg_fn = Ident::new(&format!("bg_{f}"), Span::call_site());
            let bg_fn = method_name(bg_fn, &opts);
            let underline_fn = Ident::new(&format!("underline_{f}"), Span::call_site());
            let underline_fn = method_name(underline_fn, &opts);
            quote! {
                fn #fg_fn(self) -> T;
                fn #bg_fn(self) -> T;
                fn #underline_fn(self) -> T;
            }
        })
        .collect();

    let color_ext_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let color_fn = Ident::new(&f.to_string(), Span::call_site());
            let color_fn = method_name(color_fn, &opts);

            quote! {
                fn #color_fn() -> Self;
            }
        })
        .collect();

    let style_ext_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&f.to_string(), Span::call_site());
            let style_fn = method_name(style_fn, &opts);

            quote! {
                fn #style_fn() -> Self;
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
                        let method = method_name(ident.clone(), &opts);
                        Some(quote! {
                            pub fn #method() -> #ty {
                                use #rooibos_theme::SetTheme;
                                #struct_name::with_theme(|t| t.#ident.clone())
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
            let fg_fn = method_name(fg_fn, &opts);
            let bg_fn = Ident::new(&format!("bg_{f}"), Span::call_site());
            let bg_fn = method_name(bg_fn, &opts);
            let underline_fn = Ident::new(&format!("underline_{f}"), Span::call_site());
            let underline_fn = method_name(underline_fn, &opts);

            quote! {
                fn #fg_fn(self) -> T {
                    use #rooibos_theme::SetTheme;
                    #struct_name::with_theme(|t| self.fg(t.#f))
                }

                fn #bg_fn(self) -> T {
                    use #rooibos_theme::SetTheme;
                    #struct_name::with_theme(|t| self.bg(t.#f))
                }

                fn #underline_fn(self) -> T {
                    use #rooibos_theme::SetTheme;
                    #struct_name::with_theme(|t| self.underline_color(t.#f))
                }

            }
        })
        .collect();

    let color_ext_impl_fns: TokenStream = color_fields
        .iter()
        .map(|f| {
            let color_fn = Ident::new(&f.to_string(), Span::call_site());
            let color_fn = method_name(color_fn, &opts);

            quote! {
                fn #color_fn() -> Self {
                    use #rooibos_theme::SetTheme;
                    #struct_name::with_theme(|t| t.#f.into())
                }
            }
        })
        .collect();

    let style_ext_impl_fns: TokenStream = style_fields
        .iter()
        .map(|f| {
            let style_fn = Ident::new(&f.to_string(), Span::call_site());
            let style_fn = method_name(style_fn, &opts);

            quote! {
                fn #style_fn() -> Self {
                    use #rooibos_theme::SetTheme;
                    #struct_name::with_theme(|t| t.#f.into())
                }
            }
        })
        .collect();

    Ok(quote! {
        #rooibos_theme::__local_override!(#struct_name, #global_theme, #local_theme);

        impl #rooibos_theme::SetTheme for #struct_name {
            type Theme = Self;

            fn set_local(&self) {
                #subtheme_set_local
                self.__override_set_local();
            }

            fn set_global(&self) {
                #subtheme_set_global
                self.__override_set_global();
            }

            fn unset_local() {
                #subtheme_unset_local
                Self::__override_unset_local();
            }

            fn current() -> Self {
                Self::__override_current()
            }

            fn with_theme< F, T>(f: F) -> T
            where
                F: FnOnce(&Self::Theme) -> T {
                Self::__override_with_value(f)
            }
        }

        impl #struct_name {
            #impl_fns
        }

        pub trait #style_trait<T> {
            #style_trait_fns
        }

        impl<T, U> #style_trait<T> for U
        where
            U: #rooibos_theme::Styled<Item = T>
        {
            #style_impl_fns
        }

        pub trait #color_trait<T> {
            #color_trait_fns
        }

        impl<'a, T, U> #color_trait<T> for U
        where
            U: #rooibos_theme::Stylize<'a, T>,
        {
            #color_impl_fns
        }

        pub trait #color_ext_trait {
            #color_ext_fns
        }

        impl #color_ext_trait for #rooibos_theme::Color {
            #color_ext_impl_fns
        }

        impl #color_ext_trait for #ratatui::style::Color {
            #color_ext_impl_fns
        }

        pub trait #style_ext_trait {
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
