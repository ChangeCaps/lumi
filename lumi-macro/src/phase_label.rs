use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

use crate::get_lumi;

pub fn derive_phase_label(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let lumi = get_lumi();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let impl_into = impl_into(&input.data);

    let expanded = quote! {
        impl #impl_generics ::std::convert::Into<#lumi::renderer::PhaseLabel> for #name #ty_generics #where_clause {
            fn into(self) -> #lumi::renderer::PhaseLabel {
                #impl_into
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn impl_into(data: &Data) -> TokenStream {
    let lumi = get_lumi();

    match data {
        Data::Enum(data) => {
            let variants = data.variants.iter().enumerate().map(|(i, variant)| {
                let ident = &variant.ident;

                if !matches!(variant.fields, syn::Fields::Unit) {
                    unimplemented!("PhaseLabel enum variants must be unit variants");
                }

                quote! {
                    Self::#ident => #lumi::renderer::PhaseLabel::new::<Self>(#i),
                }
            });

            quote! {
                match self {
                    #(#variants)*
                    _ => #lumi::renderer::PhaseLabel::new::<Self>(0),
                }
            }
        }
        _ => unimplemented!("PhaseLabel can only be derived for enums"),
    }
}
