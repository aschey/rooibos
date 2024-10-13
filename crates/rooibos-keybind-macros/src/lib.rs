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
        #str_tokens.parse::<TerminalKey>().expect("already validated")
    })
}

#[manyhow(proc_macro_derive(Commands))]
pub fn derive_commands(input: DeriveInput, emitter: &mut Emitter) -> manyhow::Result {
    let ident = input.ident;
    Ok(quote! {
        impl From<#ident> for Action<AppInfo<#ident>> {
            fn from(value: #ident) -> Self {
                Action::Application(value)
            }
        }

        impl ApplicationAction for #ident {
            fn is_edit_sequence(
                &self,
                ctx: &modalkit::editing::context::EditContext,
            ) -> modalkit::keybindings::SequenceStatus {
                SequenceStatus::Break
            }

            fn is_last_action(
                &self,
                ctx: &modalkit::editing::context::EditContext,
            ) -> modalkit::keybindings::SequenceStatus {
                SequenceStatus::Atom
            }

            fn is_last_selection(
                &self,
                ctx: &modalkit::editing::context::EditContext,
            ) -> modalkit::keybindings::SequenceStatus {
                SequenceStatus::Ignore
            }

            fn is_switchable(&self, ctx: &EditContext) -> bool {
                false
            }
        }

        impl CommandGenerator<#ident> for #ident {
            fn generate_commands(command_handler: &mut CommandHandler<#ident>) {
                rooibos_keybind::generate_commands(command_handler)
            }
        }

        impl CommandCompleter for #ident {
            fn complete(text: &str, cursor_position: usize) -> Vec<String> {
                rooibos_keybind::complete::<#ident>(text, cursor_position)
            }
        }
    })
}
