use manyhow::{Emitter, ErrorMessage, manyhow};
use modalkit::key::TerminalKey;
use proc_macro2::TokenStream;
use quote::quote;

#[manyhow]
#[proc_macro]
pub fn key(tokens: TokenStream, emitter: &mut Emitter) -> manyhow::Result {
    let tokens = tokens.to_string();
    let mut token_chars = tokens.chars();
    // token_chars.next();
    // token_chars.next_back();
    let str_tokens = token_chars.as_str();
    if let Err(e) = str_tokens.parse::<TerminalKey>() {
        emitter.emit(ErrorMessage::call_site(format!(
            "Invalid key combination {str_tokens}: {e:?}"
        )));
    }
    Ok(quote! {#str_tokens.parse::<TerminalKey>().expect("already validated") })
}
