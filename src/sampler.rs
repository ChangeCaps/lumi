use std::{borrow::Cow, num::NonZeroU8, ops::Deref, sync::Arc};

use wgpu::{AddressMode, CompareFunction, FilterMode, SamplerBorderColor, SamplerDescriptor};

use crate::{
    bind::{SamplerBinding, SharedBindingResource},
    bind_key::BindKey,
    id::SamplerId,
    Device, Queue,
};

#[derive(Debug)]
struct SharedSamplerInner {
    sampler: wgpu::Sampler,
    id: SamplerId,

    label: Option<Cow<'static, str>>,
    address_mode_u: AddressMode,
    address_mode_v: AddressMode,
    address_mode_w: AddressMode,
    mag_filter: FilterMode,
    min_filter: FilterMode,
    mipmap_filter: FilterMode,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: Option<CompareFunction>,
    anisotropy_clamp: Option<NonZeroU8>,
    border_color: Option<SamplerBorderColor>,
}

#[derive(Clone, Debug)]
pub struct SharedSampler {
    inner: Arc<SharedSamplerInner>,
}

impl SharedSampler {
    #[inline]
    pub fn new(sampler: wgpu::Sampler, desc: &wgpu::SamplerDescriptor) -> Self {
        Self {
            inner: Arc::new(SharedSamplerInner {
                sampler,
                id: SamplerId::new(),

                label: desc.label.map(|label| Cow::Owned(label.to_owned())),
                address_mode_u: desc.address_mode_u,
                address_mode_v: desc.address_mode_v,
                address_mode_w: desc.address_mode_w,
                mag_filter: desc.mag_filter,
                min_filter: desc.min_filter,
                mipmap_filter: desc.mipmap_filter,
                lod_min_clamp: desc.lod_min_clamp,
                lod_max_clamp: desc.lod_max_clamp,
                compare: desc.compare,
                anisotropy_clamp: desc.anisotropy_clamp,
                border_color: desc.border_color,
            }),
        }
    }

    #[inline]
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.inner.sampler
    }

    #[inline]
    pub fn id(&self) -> SamplerId {
        self.inner.id
    }

    #[inline]
    pub fn address_mode_u(&self) -> AddressMode {
        self.inner.address_mode_u
    }

    #[inline]
    pub fn address_mode_v(&self) -> AddressMode {
        self.inner.address_mode_v
    }

    #[inline]
    pub fn address_mode_w(&self) -> AddressMode {
        self.inner.address_mode_w
    }

    #[inline]
    pub fn mag_filter(&self) -> FilterMode {
        self.inner.mag_filter
    }

    #[inline]
    pub fn min_filter(&self) -> FilterMode {
        self.inner.min_filter
    }

    #[inline]
    pub fn mipmap_filter(&self) -> FilterMode {
        self.inner.mipmap_filter
    }

    #[inline]
    pub fn lod_min_clamp(&self) -> f32 {
        self.inner.lod_min_clamp
    }

    #[inline]
    pub fn lod_max_clamp(&self) -> f32 {
        self.inner.lod_max_clamp
    }

    #[inline]
    pub fn compare(&self) -> Option<CompareFunction> {
        self.inner.compare
    }

    #[inline]
    pub fn anisotropy_clamp(&self) -> Option<NonZeroU8> {
        self.inner.anisotropy_clamp
    }

    #[inline]
    pub fn border_color(&self) -> Option<SamplerBorderColor> {
        self.inner.border_color
    }

    #[inline]
    pub fn descriptor<'a>(&'a self) -> SamplerDescriptor<'a> {
        SamplerDescriptor {
            label: self.inner.label.as_deref(),
            address_mode_u: self.inner.address_mode_u,
            address_mode_v: self.inner.address_mode_v,
            address_mode_w: self.inner.address_mode_w,
            mag_filter: self.inner.mag_filter,
            min_filter: self.inner.min_filter,
            mipmap_filter: self.inner.mipmap_filter,
            lod_min_clamp: self.inner.lod_min_clamp,
            lod_max_clamp: self.inner.lod_max_clamp,
            compare: self.inner.compare,
            anisotropy_clamp: self.inner.anisotropy_clamp,
            border_color: self.inner.border_color,
        }
    }

    #[inline]
    pub fn static_descriptor(&self) -> SamplerDescriptor<'static> {
        SamplerDescriptor {
            label: None,
            address_mode_u: self.inner.address_mode_u,
            address_mode_v: self.inner.address_mode_v,
            address_mode_w: self.inner.address_mode_w,
            mag_filter: self.inner.mag_filter,
            min_filter: self.inner.min_filter,
            mipmap_filter: self.inner.mipmap_filter,
            lod_min_clamp: self.inner.lod_min_clamp,
            lod_max_clamp: self.inner.lod_max_clamp,
            compare: self.inner.compare,
            anisotropy_clamp: self.inner.anisotropy_clamp,
            border_color: self.inner.border_color,
        }
    }
}

impl Deref for SharedSampler {
    type Target = wgpu::Sampler;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.sampler()
    }
}

impl PartialEq for SharedSampler {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl Eq for SharedSampler {}

impl SamplerBinding for SharedSampler {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.id())
    }

    #[inline]
    fn binding(
        &self,
        _device: &Device,
        _queue: &Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        SharedBindingResource::Sampler(self.clone())
    }
}

impl SamplerBinding for &SharedSampler {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        SamplerBinding::bind_key(*self)
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        SamplerBinding::binding(*self, device, queue, state)
    }
}
