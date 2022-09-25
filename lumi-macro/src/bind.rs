use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    custom_keyword, parse::ParseStream, parse_macro_input, spanned::Spanned, Attribute, Data,
    DeriveInput, Error, Token,
};

use crate::get_lumi;

custom_keyword!(name);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum BindingType {
    UniformBuffer,
    StorageBuffer,
    Texture,
    StorageTexture,
    Sampler,
}

impl ToTokens for BindingType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = match self {
            Self::UniformBuffer => quote!(UniformBinding),
            Self::StorageBuffer => quote!(StorageBinding),
            Self::Texture => quote!(TextureBinding),
            Self::StorageTexture => quote!(StorageTextureBinding),
            Self::Sampler => quote!(SamplerBinding),
        };

        tokens.extend(ty);
    }
}

struct BindingAttribute {
    name: Option<String>,
    filtering: bool,
}

struct AttributeInfo {
    bindings: HashMap<BindingType, BindingAttribute>,
}

impl AttributeInfo {
    fn new(attrs: &[Attribute]) -> Result<Self, Error> {
        let mut bindings = HashMap::new();

        for attr in attrs {
            let ty = if attr.path.is_ident("uniform") {
                BindingType::UniformBuffer
            } else if attr.path.is_ident("storage_buffer") {
                BindingType::StorageBuffer
            } else if attr.path.is_ident("texture") {
                BindingType::Texture
            } else if attr.path.is_ident("storage_texture") {
                BindingType::StorageTexture
            } else if attr.path.is_ident("sampler") {
                BindingType::Sampler
            } else {
                continue;
            };

            if bindings.contains_key(&ty) {
                return Err(Error::new_spanned(
                    attr,
                    format!("duplicate binding type: {:?}", ty),
                ));
            }

            let mut name = None;
            let mut filtering = true;

            if !attr.tokens.is_empty() {
                attr.parse_args_with(|parser: ParseStream| {
                    while !parser.is_empty() {
                        let ident = parser.parse::<syn::Ident>()?;

                        if ident == "name" {
                            parser.parse::<Token![=]>()?;
                            let lit = parser.parse::<syn::LitStr>()?;
                            name = Some(lit.value());
                        } else if ident == "filtering" {
                            parser.parse::<Token![=]>()?;
                            let lit = parser.parse::<syn::LitBool>()?;
                            filtering = lit.value;
                        } else {
                            return Err(
                                parser.error(format!("unknown attribute argument: {}", ident))
                            );
                        }

                        if parser.is_empty() {
                            break;
                        }

                        parser.parse::<Token![,]>()?;
                    }

                    Ok(())
                })?;
            };

            let attr = BindingAttribute { name, filtering };

            bindings.insert(ty, attr);
        }

        Ok(Self { bindings })
    }
}

pub fn derive_bind(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let lumi = get_lumi();

    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let entries_impl = impl_entries(&input.data, &lumi);

    macro_rules! impl_bind {
        ($ty:ident, $fn:ident) => {{
            let bind_impl = impl_bind(&input.data, &lumi, BindingType::$ty);

            quote! {
                fn $fn(
                    &self,
                    device: &#lumi::SharedDevice,
                    queue: &#lumi::SharedQueue,
                    name: &::std::primitive::str,
                    state: &mut dyn ::std::any::Any,
                ) -> ::std::option::Option<#lumi::SharedBindingResource> {
                    #bind_impl
                }
            }
        }};
    }

    let uniform_impl = impl_bind!(UniformBuffer, get_uniform);
    let storage_impl = impl_bind!(StorageBuffer, get_storage);
    let texture_impl = impl_bind!(Texture, get_texture);
    let storage_texture_impl = impl_bind!(StorageTexture, get_storage_texture);
    let sampler_impl = impl_bind!(Sampler, get_sampler);

    let expanded = quote! {
        impl #impl_generics #lumi::Bind for #ident #ty_generics #where_clause {
            fn entries() -> ::std::collections::LinkedList<#lumi::BindingLayoutEntry> {
                #entries_impl
            }

            #uniform_impl
            #storage_impl
            #texture_impl
            #storage_texture_impl
            #sampler_impl
        }
    };

    expanded.into()
}

fn impl_entries(data: &Data, lumi: &syn::Path) -> TokenStream {
    match data {
        Data::Struct(data) => {
            let fields = data.fields.iter().map(|field| {
                let field_ty = &field.ty;
                let name = field.ident.clone().unwrap().to_string();

                let attrs = AttributeInfo::new(&field.attrs).unwrap();

                let mut bindings = Vec::new();
                for (ty, attr) in attrs.bindings.iter() {
                    let name = attr.name.as_ref().unwrap_or(&name);
                    let filtering = attr.filtering;

                    let entry = match ty {
                        BindingType::Sampler => quote! {
                            <#field_ty as #lumi::#ty>::entry(#filtering)
                                .into_layout_entry::<<#field_ty as #lumi::#ty>::State>(#name)
                        },
                        _ => quote! {
                            <#field_ty as #lumi::#ty>::entry()
                                .into_layout_entry::<<#field_ty as #lumi::#ty>::State>(#name)
                        },
                    };

                    let binding = quote_spanned! {field.ident.span()=>
                        entries.push_back(#entry);
                    };

                    bindings.push(binding);
                }

                quote! {
                    #(#bindings)*
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

fn impl_bind(data: &Data, lumi: &syn::Path, binding_ty: BindingType) -> TokenStream {
    match data {
        Data::Struct(data) => {
            let fields = data.fields.iter().enumerate().map(|(i, field)| {
                let ty = &field.ty;

                let index = syn::Index::from(i);
                let field_ident = match &field.ident {
                    Some(ident) => quote!(#ident),
                    None => quote!(#index),
                };

                let name = field.ident.clone().unwrap().to_string();
                let attrs = AttributeInfo::new(&field.attrs).unwrap();

                if let Some(attr) = attrs.bindings.get(&binding_ty) {
                    let name = attr.name.as_ref().unwrap_or(&name);
                    quote_spanned! {field.ident.span()=>
                        #name => Some(<#ty as #lumi::#binding_ty>::binding(
                            &self.#field_ident,
                            device,
                            queue,
                            state.downcast_mut().unwrap(),
                        )),
                    }
                } else {
                    quote!()
                }
            });

            quote! {
                match name {
                    #( #fields )*
                    _ => None,
                }
            }
        }
        _ => unimplemented!("Bind must be derived for structs"),
    }
}
