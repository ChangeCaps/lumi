use std::{
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use once_cell::sync::OnceCell;
use wgpu::TextureFormat;

use crate::{
    bind::{DefaultSampler, DefaultTexture, SamplerBinding, SharedBindingResource, TextureBinding},
    bind_key::BindKey,
    id::TextureId,
    Device, Queue, SharedSampler, SharedTexture, SharedTextureView,
};

use super::ImageData;

#[derive(Default)]
struct ImageInner {
    image: ImageData,
    texture: OnceCell<SharedTexture>,
    write: AtomicBool,
}

impl Clone for ImageInner {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
            texture: OnceCell::new(),
            write: AtomicBool::new(false),
        }
    }
}

#[derive(Clone, Default)]
pub struct Image {
    inner: Arc<ImageInner>,
}

impl Image {
    #[inline]
    pub fn new(image: ImageData) -> Self {
        Self {
            inner: Arc::new(ImageInner {
                image,
                texture: OnceCell::new(),
                write: AtomicBool::new(false),
            }),
        }
    }

    #[inline]
    pub fn render_target(width: u32, height: u32) -> Self {
        Self::new(ImageData::with_format(
            width,
            height,
            Vec::new(),
            TextureFormat::Bgra8UnormSrgb,
        ))
    }

    #[inline]
    pub fn open_srgb(path: &str) -> Result<Self, image::ImageError> {
        let image = ImageData::open_srgb(path)?;
        Ok(Self::new(image))
    }

    #[inline]
    pub fn open_hdr(path: &str) -> Result<Self, image::ImageError> {
        let image = ImageData::open_hdr(path)?;
        Ok(Self::new(image))
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut ImageInner {
        Arc::make_mut(&mut self.inner)
    }

    #[inline]
    pub fn set_texture(&mut self, texture: SharedTexture) {
        let inner = self.inner_mut();

        if let Some(inner_texture) = inner.texture.get_mut() {
            *inner_texture = texture;
        } else {
            inner
                .texture
                .set(texture)
                .expect("texture already set, you should never see this");
        }
    }

    #[inline]
    pub fn texture_id(&self) -> Option<TextureId> {
        self.get_texture().map(|texture| texture.id())
    }

    #[inline]
    pub fn get_texture(&self) -> Option<&SharedTexture> {
        self.inner.texture.get()
    }

    #[inline]
    pub fn data(&self) -> &ImageData {
        &self.inner.image
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut ImageData {
        let inner = Arc::make_mut(&mut self.inner);
        *inner.write.get_mut() = true;
        &mut inner.image
    }

    #[inline]
    pub fn texture_view(&self, device: &Device, queue: &Queue) -> SharedTextureView {
        let texture = self
            .inner
            .texture
            .get_or_init(|| self.inner.image.create_texture(device, queue));

        if self.inner.write.swap(false, Ordering::SeqCst) {
            self.inner.image.write_texture(queue, &texture);
        }

        texture.create_view(&Default::default())
    }
}

impl From<ImageData> for Image {
    #[inline]
    fn from(image: ImageData) -> Self {
        Self::new(image)
    }
}

impl std::fmt::Debug for Image {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedImage")
            .field("image", &self.inner.image)
            .finish()
    }
}

impl Deref for Image {
    type Target = ImageData;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl DerefMut for Image {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

impl TextureBinding for Image {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        TextureBinding::bind_key(self.data())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        SharedBindingResource::TextureView(self.texture_view(device, queue))
    }
}

impl SamplerBinding for Image {
    type State = Option<SharedSampler>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        SamplerBinding::bind_key(self.data())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        SamplerBinding::binding(&self.inner.image, device, queue, state)
    }
}

impl DefaultTexture for Image {
    fn default_texture(device: &Device, queue: &Queue) -> SharedTextureView {
        ImageData::default_texture(device, queue)
    }
}

impl DefaultSampler for Image {
    fn default_sampler(device: &Device, queue: &Queue) -> SharedSampler {
        ImageData::default_sampler(device, queue)
    }
}
