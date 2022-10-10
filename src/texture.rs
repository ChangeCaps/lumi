use std::{ops::Deref, sync::Arc};

use once_cell::sync::OnceCell;
use wgpu::{
    Device, Extent3d, FilterMode, Queue, SamplerDescriptor, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};

use crate::{
    bind::{SamplerBinding, SharedBindingResource, StorageTextureBinding, TextureBinding},
    bind_key::BindKey,
    id::{TextureId, TextureViewId},
    SharedDevice, SharedSampler,
};

#[derive(Debug)]
struct SharedTextureInner {
    texture: wgpu::Texture,
    id: TextureId,

    size: Extent3d,
    mip_level_count: u32,
    sample_count: u32,
    dimension: TextureDimension,
    format: TextureFormat,
    usage: TextureUsages,
}

#[derive(Clone, Debug)]
pub struct SharedTexture {
    inner: Arc<SharedTextureInner>,
}

impl SharedTexture {
    #[inline]
    pub fn new(texture: wgpu::Texture, desc: &TextureDescriptor) -> Self {
        Self {
            inner: Arc::new(SharedTextureInner {
                texture,
                id: TextureId::new(),

                size: desc.size,
                mip_level_count: desc.mip_level_count,
                sample_count: desc.sample_count,
                dimension: desc.dimension,
                format: desc.format,
                usage: desc.usage,
            }),
        }
    }

    #[inline]
    pub fn texture(&self) -> &wgpu::Texture {
        &self.inner.texture
    }

    #[inline]
    pub fn id(&self) -> TextureId {
        self.inner.id
    }

    #[inline]
    pub fn size(&self) -> Extent3d {
        self.inner.size
    }

    #[inline]
    pub fn mip_level_count(&self) -> u32 {
        self.inner.mip_level_count
    }

    #[inline]
    pub fn sample_count(&self) -> u32 {
        self.inner.sample_count
    }

    #[inline]
    pub fn dimension(&self) -> TextureDimension {
        self.inner.dimension
    }

    #[inline]
    pub fn format(&self) -> TextureFormat {
        self.inner.format
    }

    #[inline]
    pub fn usage(&self) -> TextureUsages {
        self.inner.usage
    }

    #[inline]
    pub fn desc(&self) -> TextureDescriptor {
        TextureDescriptor {
            label: None,
            size: self.inner.size,
            mip_level_count: self.inner.mip_level_count,
            sample_count: self.inner.sample_count,
            dimension: self.inner.dimension,
            format: self.inner.format,
            usage: self.inner.usage,
        }
    }

    #[inline]
    pub fn create_view(&self, desc: &TextureViewDescriptor) -> SharedTextureView {
        SharedTextureView::new(self.inner.texture.create_view(desc), &self.desc(), desc)
    }
}

impl Deref for SharedTexture {
    type Target = wgpu::Texture;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.texture()
    }
}

impl PartialEq for SharedTexture {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for SharedTexture {}

#[derive(Debug)]
struct SharedTextureViewInner {
    view: wgpu::TextureView,
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

#[derive(Clone, Debug)]
pub struct SharedTextureView {
    inner: Arc<SharedTextureViewInner>,
}

impl SharedTextureView {
    #[inline]
    pub fn new(
        view: wgpu::TextureView,
        texture_desc: &TextureDescriptor,
        view_desc: &TextureViewDescriptor,
    ) -> Self {
        Self {
            inner: Arc::new(SharedTextureViewInner {
                view,
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
            }),
        }
    }

    #[inline]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.inner.view
    }

    #[inline]
    pub fn id(&self) -> TextureViewId {
        self.inner.id
    }

    #[inline]
    pub fn size(&self) -> Extent3d {
        self.inner.size
    }

    #[inline]
    pub fn mip_level_count(&self) -> u32 {
        self.inner.mip_level_count
    }

    #[inline]
    pub fn sample_count(&self) -> u32 {
        self.inner.sample_count
    }

    #[inline]
    pub fn dimension(&self) -> TextureDimension {
        self.inner.dimension
    }

    #[inline]
    pub fn format(&self) -> TextureFormat {
        self.inner.format
    }

    #[inline]
    pub fn usage(&self) -> TextureUsages {
        self.inner.usage
    }

    #[inline]
    pub fn aspect(&self) -> TextureAspect {
        self.inner.aspect
    }

    #[inline]
    pub fn base_mip_level(&self) -> u32 {
        self.inner.base_mip_level
    }

    #[inline]
    pub fn base_array_layer(&self) -> u32 {
        self.inner.base_array_layer
    }
}

impl Deref for SharedTextureView {
    type Target = wgpu::TextureView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.view()
    }
}

impl PartialEq for SharedTextureView {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for SharedTextureView {}

impl TextureBinding for SharedTextureView {
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
        SharedBindingResource::TextureView(self.clone())
    }
}

impl TextureBinding for &SharedTextureView {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.id())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        TextureBinding::binding(*self, device, queue, state)
    }
}

impl StorageTextureBinding for SharedTextureView {
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
        SharedBindingResource::TextureView(self.clone())
    }
}

impl StorageTextureBinding for &SharedTextureView {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.id())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        StorageTextureBinding::binding(*self, device, queue, state)
    }
}

impl SamplerBinding for SharedTextureView {
    type State = OnceCell<SharedSampler>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.id())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        _queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        let sampler = state.get_or_init(|| {
            device.create_shared_sampler(&SamplerDescriptor {
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                ..Default::default()
            })
        });

        SharedBindingResource::Sampler(sampler.clone())
    }
}

impl SamplerBinding for &SharedTextureView {
    type State = OnceCell<SharedSampler>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.id())
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
