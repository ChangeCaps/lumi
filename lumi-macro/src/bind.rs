use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    custom_keyword,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Error, Token,
};

use crate::get_lumi;

custom_keyword!(name);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TextureSampleType {
    Float { filterable: bool },
    Depth,
    Uint,
    Sint,
}

impl Parse for TextureSampleType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "float" => Ok(Self::Float { filterable: false }),
            "float_filterable" => Ok(Self::Float { filterable: true }),
            "depth" => Ok(Self::Depth),
            "uint" => Ok(Self::Uint),
            "sint" => Ok(Self::Sint),
            _ => Err(Error::new(ident.span(), "invalid texture sample type")),
        }
    }
}

impl ToTokens for TextureSampleType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let lumi = get_lumi();

        match self {
            Self::Float { filterable } => {
                if *filterable {
                    tokens.extend(quote! { #lumi::TextureSampleType::Float { filterable: true } });
                } else {
                    tokens.extend(quote! { #lumi::TextureSampleType::Float { filterable: false } });
                }
            }
            Self::Depth => tokens.extend(quote! { #lumi::TextureSampleType::Depth }),
            Self::Uint => tokens.extend(quote! { #lumi::TextureSampleType::Uint }),
            Self::Sint => tokens.extend(quote! { #lumi::TextureSampleType::Sint }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TextureViewDimension {
    D1,
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

impl Parse for TextureViewDimension {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "d1" => Ok(Self::D1),
            "d2" => Ok(Self::D2),
            "d2_array" => Ok(Self::D2Array),
            "cube" => Ok(Self::Cube),
            "cube_array" => Ok(Self::CubeArray),
            "d3" => Ok(Self::D3),
            _ => Err(Error::new(ident.span(), "invalid texture view dimension")),
        }
    }
}

impl ToTokens for TextureViewDimension {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let lumi = get_lumi();

        match self {
            Self::D1 => tokens.extend(quote! { #lumi::TextureViewDimension::D1 }),
            Self::D2 => tokens.extend(quote! { #lumi::TextureViewDimension::D2 }),
            Self::D2Array => tokens.extend(quote! { #lumi::TextureViewDimension::D2Array }),
            Self::Cube => tokens.extend(quote! { #lumi::TextureViewDimension::Cube }),
            Self::CubeArray => tokens.extend(quote! { #lumi::TextureViewDimension::CubeArray }),
            Self::D3 => tokens.extend(quote! { #lumi::TextureViewDimension::D3 }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TexelFormat {
    Rgba8Unorm,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Float,
    R32Uint,
    R32Sint,
    R32Float,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
}

impl Parse for TexelFormat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "rgba8_unorm" => Ok(Self::Rgba8Unorm),
            "rgba8_snorm" => Ok(Self::Rgba8Snorm),
            "rgba8_uint" => Ok(Self::Rgba8Uint),
            "rgba8_sint" => Ok(Self::Rgba8Sint),
            "rgba16_uint" => Ok(Self::Rgba16Uint),
            "rgba16_sint" => Ok(Self::Rgba16Sint),
            "rgba16_float" => Ok(Self::Rgba16Float),
            "r32_uint" => Ok(Self::R32Uint),
            "r32_sint" => Ok(Self::R32Sint),
            "r32_float" => Ok(Self::R32Float),
            "rg32_uint" => Ok(Self::Rg32Uint),
            "rg32_sint" => Ok(Self::Rg32Sint),
            "rg32_float" => Ok(Self::Rg32Float),
            "rgba32_uint" => Ok(Self::Rgba32Uint),
            "rgba32_sint" => Ok(Self::Rgba32Sint),
            "rgba32_float" => Ok(Self::Rgba32Float),
            _ => Err(Error::new(ident.span(), "invalid texel format")),
        }
    }
}

impl ToTokens for TexelFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let lumi = get_lumi();

        match self {
            Self::Rgba8Unorm => tokens.extend(quote! { #lumi::TextureFormat::Rgba8Unorm }),
            Self::Rgba8Snorm => tokens.extend(quote! { #lumi::TextureFormat::Rgba8Snorm }),
            Self::Rgba8Uint => tokens.extend(quote! { #lumi::TextureFormat::Rgba8Uint }),
            Self::Rgba8Sint => tokens.extend(quote! { #lumi::TextureFormat::Rgba8Sint }),
            Self::Rgba16Uint => tokens.extend(quote! { #lumi::TextureFormat::Rgba16Uint }),
            Self::Rgba16Sint => tokens.extend(quote! { #lumi::TextureFormat::Rgba16Sint }),
            Self::Rgba16Float => tokens.extend(quote! { #lumi::TextureFormat::Rgba16Float }),
            Self::R32Uint => tokens.extend(quote! { #lumi::TextureFormat::R32Uint }),
            Self::R32Sint => tokens.extend(quote! { #lumi::TextureFormat::R32Sint }),
            Self::R32Float => tokens.extend(quote! { #lumi::TextureFormat::R32Float }),
            Self::Rg32Uint => tokens.extend(quote! { #lumi::TextureFormat::Rg32Uint }),
            Self::Rg32Sint => tokens.extend(quote! { #lumi::TextureFormat::Rg32Sint }),
            Self::Rg32Float => tokens.extend(quote! { #lumi::TextureFormat::Rg32Float }),
            Self::Rgba32Uint => tokens.extend(quote! { #lumi::TextureFormat::Rgba32Uint }),
            Self::Rgba32Sint => tokens.extend(quote! { #lumi::TextureFormat::Rgba32Sint }),
            Self::Rgba32Float => tokens.extend(quote! { #lumi::TextureFormat::Rgba32Float }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum BindingType {
    UniformBuffer,
    StorageBuffer,
    Texture,
    StorageTexture,
    Sampler,
}

impl BindingType {
    pub fn is_storage(&self) -> bool {
        matches!(self, Self::StorageBuffer | Self::StorageTexture)
    }

    pub fn is_texture(&self) -> bool {
        matches!(self, Self::Texture | Self::StorageTexture)
    }
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

#[derive(Default)]
struct BindingAttribute {
    name: Option<String>,
    read_only: Option<bool>,
    sample_type: Option<TextureSampleType>,
    view_dimension: Option<TextureViewDimension>,
    multisampled: Option<bool>,
    texel_format: Option<TexelFormat>,
    filtering: Option<bool>,
}

impl BindingAttribute {
    fn changes(&self) -> Vec<TokenStream> {
        let lumi = get_lumi();
        let mut changes = Vec::new();

        if let Some(read_only) = self.read_only {
            changes.push(quote! {match &mut entry.ty {
                #lumi::BindingType::StorageTexture { read_only, .. } => *read_only = #read_only,
                _ => {}
            }});
        }

        if let Some(sample_type) = self.sample_type {
            changes.push(quote! {match &mut entry.ty {
                #lumi::BindingType::Texture { sample_type, .. } => *sample_type = #sample_type,
                _ => {}
            }});
        }

        if let Some(view_dimension) = self.view_dimension {
            changes.push(quote! {match &mut entry.ty {
                #lumi::BindingType::Texture { view_dimension, .. } => *view_dimension = #view_dimension,
                _ => {}
            }});
        }

        if let Some(multisampled) = self.multisampled {
            changes.push(quote! {match &mut entry.ty {
                #lumi::BindingType::Texture { multisampled, .. } => *multisampled = #multisampled,
                _ => {}
            }});
        }

        if let Some(texel_format) = self.texel_format {
            changes.push(quote! {match &mut entry.ty {
                #lumi::BindingType::StorageTexture { format, .. } => *format = #texel_format,
                _ => {}
            }});
        }

        if let Some(filtering) = self.filtering {
            let ty = if filtering {
                quote!(#lumi::SamplerBindingType::Filtering)
            } else {
                quote!(#lumi::SamplerBindingType::NonFiltering)
            };

            changes.push(quote! {match &mut entry.ty {
                #lumi::BindingType::Sampler(ty) => *ty = #ty,
                _ => {}
            }});
        }

        changes
    }
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

            let mut binding = BindingAttribute::default();

            if !attr.tokens.is_empty() {
                attr.parse_args_with(|parser: ParseStream| {
                    while !parser.is_empty() {
                        let ident = parser.parse::<syn::Ident>()?;

                        match ident.to_string().as_str() {
                            "name" => {
                                parser.parse::<Token![=]>()?;
                                let name = parser.parse::<syn::LitStr>()?;
                                binding.name = Some(name.value());
                            }
                            "read_only" if ty.is_storage() => {
                                parser.parse::<Token![=]>()?;
                                let read_only = parser.parse::<syn::LitBool>()?;
                                binding.read_only = Some(read_only.value);
                            }
                            "sample_type" if ty == BindingType::Texture => {
                                parser.parse::<Token![=]>()?;
                                let sample_type = parser.parse::<TextureSampleType>()?;
                                binding.sample_type = Some(sample_type);
                            }
                            "dimension" if ty.is_texture() => {
                                parser.parse::<Token![=]>()?;
                                let view_dimension = parser.parse::<TextureViewDimension>()?;
                                binding.view_dimension = Some(view_dimension);
                            }
                            "multisampled" if ty.is_texture() => {
                                parser.parse::<Token![=]>()?;
                                let multisampled = parser.parse::<syn::LitBool>()?;
                                binding.multisampled = Some(multisampled.value);
                            }
                            "texel_format" if ty == BindingType::StorageTexture => {
                                parser.parse::<Token![=]>()?;
                                let texel_format = parser.parse::<TexelFormat>()?;
                                binding.texel_format = Some(texel_format);
                            }
                            "filtering" if ty == BindingType::Sampler => {
                                parser.parse::<Token![=]>()?;
                                let filtering = parser.parse::<syn::LitBool>()?;
                                binding.filtering = Some(filtering.value);
                            }
                            _ => return Err(Error::new(ident.span(), "invalid binding attribute")),
                        }

                        if parser.is_empty() {
                            break;
                        }

                        parser.parse::<Token![,]>()?;
                    }

                    Ok(())
                })?;
            };

            bindings.insert(ty, binding);
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
                    device: &#lumi::Device,
                    queue: &#lumi::Queue,
                    name: &::std::primitive::str,
                    state: &mut dyn ::std::any::Any,
                ) -> ::std::option::Option<#lumi::bind::SharedBindingResource> {
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
        impl #impl_generics #lumi::bind::Bind for #ident #ty_generics #where_clause {
            fn entries() -> ::std::collections::LinkedList<#lumi::bind::BindingLayoutEntry> {
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

                    let changes = attr.changes();

                    let entry = quote! {{
                        let mut entry = <#field_ty as #lumi::bind::#ty>::entry();
                        #(#changes)*
                        entry.into_layout_entry::<<#field_ty as #lumi::bind::#ty>::State>(#name)
                    }};

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
                        #name => Some(<#ty as #lumi::bind::#binding_ty>::binding(
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
