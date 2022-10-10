use proc_macro2::{Ident, Span};
use syn::parse_quote;

mod bind;
mod phase_label;

fn get_lumi() -> syn::Path {
    match proc_macro_crate::crate_name("lumi") {
        Ok(name) => match name {
            proc_macro_crate::FoundCrate::Itself => parse_quote!(crate),
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident: Ident = Ident::new(&name, Span::call_site());
                parse_quote!(::#ident)
            }
        },
        Err(_) => parse_quote!(::lumi),
    }
}

#[proc_macro_derive(
    Bind,
    attributes(uniform, storage_buffer, texture, storage_texture, sampler)
)]
pub fn derive_bind(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bind::derive_bind(input)
}

#[proc_macro_derive(PhaseLabel)]
pub fn derive_phase_label(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    phase_label::derive_phase_label(input)
}
