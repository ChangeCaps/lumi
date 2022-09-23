use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, LitInt};

use crate::get_lumi;

pub fn derive_bind(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let lumi = get_lumi();

    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let entries_impl = impl_entries(&input.data, &lumi);
    let bind_impl = impl_bind(&input.data, &lumi);

    let expanded = quote! {
        impl #impl_generics #lumi::Bind for #ident #ty_generics #where_clause {
            fn entries() -> ::std::collections::LinkedList<#lumi::BindingLayoutEntry> {
                #entries_impl
            }

            fn bindings(&self, device: &#lumi::SharedDevice) -> ::std::vec::Vec<#lumi::SharedBinding> {
                #bind_impl
            }
        }
    };

    expanded.into()
}

fn get_group(attrs: &[Attribute]) -> u32 {
    for attr in attrs {
        if attr.path.is_ident("group") {
            return attr.parse_args::<LitInt>().unwrap().base10_parse().unwrap();
        }
    }

    panic!("#[group] not defined")
}

fn impl_entries(data: &Data, lumi: &syn::Path) -> TokenStream {
    match data {
        Data::Struct(data) => {
            let fields = data.fields.iter().map(|field| {
                let ty = &field.ty;
                let group = get_group(&field.attrs);

                let name = field.ident.clone().unwrap().to_string();

                quote! {
                    entries.push_back(<#ty as #lumi::AsBinding>::entry(
                        ::std::borrow::Cow::Borrowed(#name),
                        #group,
                    ));
                }
            });

            quote! {
                let mut entries = ::std::collections::LinkedList::new();

                #( #fields )*

                entries
            }
        }
        _ => unimplemented!("Bind must be derived for structs"),
    }
}

fn impl_bind(data: &Data, lumi: &syn::Path) -> TokenStream {
    match data {
        Data::Struct(data) => {
            let fields = data.fields.iter().enumerate().map(|(i, field)| {
                let index = syn::Index::from(i);

                let ident = match &field.ident {
                    Some(ident) => quote!(#ident),
                    None => quote!(#index),
                };

                let name = field.ident.clone().unwrap().to_string();
                let group = get_group(&field.attrs);

                quote! {
                    bindings.push(#lumi::AsBinding::as_binding(
                        &self.#ident,
                        device,
                        ::std::borrow::Cow::Borrowed(#name),
                        #group
                    ));
                }
            });

            quote! {
                let mut bindings = ::std::vec::Vec::new();

                #( #fields )*

                bindings
            }
        }
        _ => unimplemented!("Bind must be derived for structs"),
    }
}
