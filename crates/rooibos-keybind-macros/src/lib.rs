use manyhow::{Emitter, ErrorMessage, manyhow};
use modalkit::key::TerminalKey;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[manyhow]
#[proc_macro]
pub fn key(tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    let tokens = tokens.to_string();
    let token_chars = tokens.chars();

    let str_tokens = token_chars.as_str();
    if let Err(e) = str_tokens.parse::<TerminalKey>() {
        emitter.emit(ErrorMessage::call_site(format!(
            "Invalid key combination {str_tokens}: {e:?}"
        )));
    }
    Ok(quote! {
        #str_tokens.parse::<rooibos::keybind::TerminalKey>().expect("already validated")
    })
}

#[manyhow(proc_macro_derive(Commands))]
pub fn derive_commands(input: DeriveInput) -> manyhow::Result {
    let ident = input.ident;
    Ok(quote! {
        impl From<#ident> for rooibos::keybind::Action<AppInfo<#ident>> {
            fn from(value: #ident) -> Self {
                rooibos::keybind::Action::Application(value)
            }
        }

        impl rooibos::keybind::ApplicationAction for #ident {
            fn is_edit_sequence(
                &self,
                ctx: &rooibos::keybind::EditContext,
            ) -> rooibos::keybind::SequenceStatus {
                rooibos::keybind::SequenceStatus::Break
            }

            fn is_last_action(
                &self,
                ctx: &rooibos::keybind::EditContext,
            ) -> rooibos::keybind::SequenceStatus {
                 rooibos::keybind::SequenceStatus::Atom
            }

            fn is_last_selection(
                &self,
                ctx: &rooibos::keybind::EditContext,
            ) -> rooibos::keybind::SequenceStatus {
                 rooibos::keybind::SequenceStatus::Ignore
            }

            fn is_switchable(&self, ctx: &rooibos::keybind::EditContext) -> bool {
                false
            }
        }

        impl CommandGenerator<#ident> for #ident {
            fn generate_commands(command_handler: &mut CommandHandler<#ident>) {
                rooibos::keybind::generate_commands(command_handler)
            }
        }

        impl CommandCompleter for #ident {
            fn complete(text: &str, cursor_position: usize) -> Vec<String> {
                rooibos::keybind::complete::<#ident>(text, cursor_position)
            }
        }
    })
}
