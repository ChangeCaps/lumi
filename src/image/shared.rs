use std::{
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use once_cell::sync::OnceCell;

use crate::{
    DefaultSampler, DefaultTexture, ImageData, SamplerBinding, SharedBindingResource, SharedDevice,
    SharedQueue, SharedSampler, SharedTexture, SharedTextureView, TextureBinding,
};

#[derive(Default)]
struct ImageInner {
    image: ImageData,
    view: OnceCell<SharedTexture>,
    write: AtomicBool,
}

impl Clone for ImageInner {
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
            view: OnceCell::new(),
            write: AtomicBool::new(false),
        }
    }
}

#[derive(Clone, Default)]
pub struct Image {
    inner: Arc<ImageInner>,
}

impl Image {
    pub fn new(image: ImageData) -> Self {
        Self {
            inner: Arc::new(ImageInner {
                image,
                view: OnceCell::new(),
                write: AtomicBool::new(false),
            }),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, image::ImageError> {
        let image = ImageData::load_from_file(path)?;
        Ok(Self::new(image))
    }

    pub fn data(&self) -> &ImageData {
        &self.inner.image
    }

    pub fn data_mut(&mut self) -> &mut ImageData {
        let inner = Arc::make_mut(&mut self.inner);
        *inner.write.get_mut() = true;
        &mut inner.image
    }
}

impl From<ImageData> for Image {
    fn from(image: ImageData) -> Self {
        Self::new(image)
    }
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedImage")
            .field("image", &self.inner.image)
            .finish()
    }
}

impl Deref for Image {
    type Target = ImageData;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

impl TextureBinding for Image {
    type State = ();

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        let texture = self
            .inner
            .view
            .get_or_init(|| self.inner.image.create_texture(device, queue))
            .clone();

        if self.inner.write.swap(false, Ordering::SeqCst) {
            self.inner.image.write_texture(queue, &texture);
        }

        SharedBindingResource::TextureView(texture.create_view(&Default::default()))
    }
}

impl SamplerBinding for Image {
    type State = Option<SharedSampler>;

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        SamplerBinding::binding(&self.inner.image, device, queue, state)
    }
}

impl DefaultTexture for Image {
    fn default_texture(device: &SharedDevice, queue: &SharedQueue) -> SharedTextureView {
        ImageData::default_texture(device, queue)
    }
}

impl DefaultSampler for Image {
    fn default_sampler(device: &SharedDevice, queue: &SharedQueue) -> SharedSampler {
        ImageData::default_sampler(device, queue)
    }
}
