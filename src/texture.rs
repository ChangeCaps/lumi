use std::{ops::Deref, sync::Arc};

use once_cell::sync::OnceCell;
use wgpu::{
    Extent3d, FilterMode, SamplerDescriptor, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};

use crate::{
    SamplerBinding, SharedBindingResource, SharedDevice, SharedQueue, SharedSampler,
    TextureBinding, TextureId, TextureViewId,
};

#[derive(Clone, Debug)]
pub struct SharedTexture {
    texture: Arc<wgpu::Texture>,
    id: TextureId,

    size: Extent3d,
    mip_level_count: u32,
    sample_count: u32,
    dimension: TextureDimension,
    format: TextureFormat,
    usage: TextureUsages,
}

impl SharedTexture {
    pub fn new(texture: wgpu::Texture, desc: &TextureDescriptor) -> Self {
        Self {
            texture: Arc::new(texture),
            id: TextureId::new(),
            size: desc.size,
            mip_level_count: desc.mip_level_count,
            sample_count: desc.sample_count,
            dimension: desc.dimension,
            format: desc.format,
            usage: desc.usage,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn id(&self) -> TextureId {
        self.id
    }

    pub fn size(&self) -> Extent3d {
        self.size
    }

    pub fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn dimension(&self) -> TextureDimension {
        self.dimension
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn usage(&self) -> TextureUsages {
        self.usage
    }

    pub fn desc(&self) -> TextureDescriptor {
        TextureDescriptor {
            label: None,
            size: self.size,
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format,
            usage: self.usage,
        }
    }

    pub fn create_view(&self, desc: &TextureViewDescriptor) -> SharedTextureView {
        SharedTextureView::new(self.texture.create_view(desc), &self.desc(), desc)
    }
}

impl Deref for SharedTexture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        self.texture()
    }
}

impl PartialEq for SharedTexture {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for SharedTexture {}

#[derive(Clone, Debug)]
pub struct SharedTextureView {
    view: Arc<wgpu::TextureView>,
    id: TextureViewId,

    size: Extent3d,
    mip_level_count: u32,
    sample_count: u32,
    dimension: TextureDimension,
    format: TextureFormat,
    usage: TextureUsages,
    aspect: TextureAspect,
    base_mip_level: u32,
    base_array_layer: u32,
}

impl SharedTextureView {
    pub fn new(
        view: wgpu::TextureView,
        texture_desc: &TextureDescriptor,
        view_desc: &TextureViewDescriptor,
    ) -> Self {
        Self {
            view: Arc::new(view),
            id: TextureViewId::new(),
            size: texture_desc.size,
            mip_level_count: texture_desc.mip_level_count,
            sample_count: texture_desc.sample_count,
            dimension: texture_desc.dimension,
            format: texture_desc.format,
            usage: texture_desc.usage,
            aspect: view_desc.aspect,
            base_mip_level: view_desc.base_mip_level,
            base_array_layer: view_desc.base_array_layer,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn id(&self) -> TextureViewId {
        self.id
    }

    pub fn size(&self) -> Extent3d {
        self.size
    }

    pub fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn dimension(&self) -> TextureDimension {
        self.dimension
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn usage(&self) -> TextureUsages {
        self.usage
    }

    pub fn aspect(&self) -> TextureAspect {
        self.aspect
    }

    pub fn base_mip_level(&self) -> u32 {
        self.base_mip_level
    }

    pub fn base_array_layer(&self) -> u32 {
        self.base_array_layer
    }
}

impl Deref for SharedTextureView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        self.view()
    }
}

impl PartialEq for SharedTextureView {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for SharedTextureView {}

impl TextureBinding for SharedTextureView {
    type State = ();

    fn binding(
        &self,
        _device: &SharedDevice,
        _queue: &SharedQueue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        SharedBindingResource::TextureView(self.clone())
    }
}

impl TextureBinding for &SharedTextureView {
    type State = ();

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        TextureBinding::binding(*self, device, queue, state)
    }
}

impl SamplerBinding for SharedTextureView {
    type State = OnceCell<SharedSampler>;

    fn binding(
        &self,
        device: &SharedDevice,
        _queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        let sampler = state.get_or_init(|| {
            device.create_shared_sampler(&SamplerDescriptor {
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..Default::default()
            })
        });

        SharedBindingResource::Sampler(sampler.clone())
    }
}

impl SamplerBinding for &SharedTextureView {
    type State = OnceCell<SharedSampler>;

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        SamplerBinding::binding(*self, device, queue, state)
    }
}
