use manyhow::manyhow;
use quote::quote;
use syn::DeriveInput;

#[manyhow(proc_macro_derive(Commands))]
pub fn derive_commands(input: DeriveInput) -> manyhow::Result {
    let ident = input.ident;
    Ok(quote! {
        impl From<#ident> for rooibos::keybind::Action<rooibos::keybind::AppInfo<#ident>> {
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

        impl rooibos::keybind::CommandGenerator<#ident> for #ident {
            fn generate_commands(command_handler: &mut rooibos::keybind::CommandHandler<#ident>) {
                rooibos::keybind::generate_commands(command_handler)
            }
        }
    })
}
