use std::{num::NonZeroU8, ops::Deref, sync::Arc};

use wgpu::{AddressMode, CompareFunction, FilterMode, SamplerBorderColor};

use crate::{
    bind::{SamplerBinding, SharedBindingResource},
    id::SamplerId,
    Device, Queue,
};

#[derive(Clone, Debug)]
pub struct SharedSampler {
    sampler: Arc<wgpu::Sampler>,
    id: SamplerId,

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

impl SharedSampler {
    pub fn new(sampler: wgpu::Sampler, desc: &wgpu::SamplerDescriptor) -> Self {
        Self {
            sampler: Arc::new(sampler),
            id: SamplerId::new(),

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
        }
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn id(&self) -> SamplerId {
        self.id
    }

    pub fn address_mode_u(&self) -> AddressMode {
        self.address_mode_u
    }

    pub fn address_mode_v(&self) -> AddressMode {
        self.address_mode_v
    }

    pub fn address_mode_w(&self) -> AddressMode {
        self.address_mode_w
    }

    pub fn mag_filter(&self) -> FilterMode {
        self.mag_filter
    }

    pub fn min_filter(&self) -> FilterMode {
        self.min_filter
    }

    pub fn mipmap_filter(&self) -> FilterMode {
        self.mipmap_filter
    }

    pub fn lod_min_clamp(&self) -> f32 {
        self.lod_min_clamp
    }

    pub fn lod_max_clamp(&self) -> f32 {
        self.lod_max_clamp
    }

    pub fn compare(&self) -> Option<CompareFunction> {
        self.compare
    }

    pub fn anisotropy_clamp(&self) -> Option<NonZeroU8> {
        self.anisotropy_clamp
    }

    pub fn border_color(&self) -> Option<SamplerBorderColor> {
        self.border_color
    }
}

impl Deref for SharedSampler {
    type Target = wgpu::Sampler;

    fn deref(&self) -> &Self::Target {
        self.sampler()
    }
}

impl PartialEq for SharedSampler {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SharedSampler {}

impl SamplerBinding for SharedSampler {
    type State = ();

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

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        SamplerBinding::binding(*self, device, queue, state)
    }
}
