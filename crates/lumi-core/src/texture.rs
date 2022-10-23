use std::{ops::Deref, sync::Arc};

use lumi_id::Id;
use wgpu::{
    Extent3d, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor,
};

pub type TextureId = Id<wgpu::Texture>;
pub type TextureViewId = Id<wgpu::TextureView>;

#[derive(Debug)]
struct SharedTextureInner {
    texture: wgpu::Texture,
    id: Id<wgpu::Texture>,

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
                id: Id::new(),

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
    pub fn id(&self) -> Id<wgpu::Texture> {
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
    id: Id<wgpu::TextureView>,

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
                id: Id::new(),

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
    pub fn id(&self) -> Id<wgpu::TextureView> {
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
