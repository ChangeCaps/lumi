use std::ops::{Deref, DerefMut};

use wgpu::TextureFormat;

use crate::{
    bind::{DefaultSampler, DefaultTexture, SamplerBinding, SharedBindingResource, TextureBinding},
    bind_key::BindKey,
    Device, Queue, SharedSampler, SharedTextureView,
};

use super::{Image, ImageData};

#[derive(Clone, Debug, Default)]
pub struct NormalMap(Image);

impl NormalMap {
    pub fn new(mut image: Image) -> Self {
        match image.format {
            TextureFormat::Rgba8UnormSrgb => {
                image.format = TextureFormat::Rgba8Unorm;
            }
            _ => {}
        }

        Self(image)
    }

    #[inline]
    #[cfg(feature = "image")]
    pub fn open(path: &str) -> Result<Self, image::ImageError> {
        let image = Image::open_srgb(path)?;
        Ok(image.into())
    }

    pub fn image(&self) -> &Image {
        &self.0
    }

    pub fn data(&self) -> &ImageData {
        self.0.data()
    }

    pub fn data_mut(&mut self) -> &mut ImageData {
        self.0.data_mut()
    }
}

impl From<ImageData> for NormalMap {
    fn from(image_data: ImageData) -> Self {
        Self::new(Image::new(image_data))
    }
}

impl From<Image> for NormalMap {
    fn from(image: Image) -> Self {
        Self::new(image)
    }
}

impl From<NormalMap> for Image {
    fn from(normal_map: NormalMap) -> Self {
        normal_map.0
    }
}

impl Deref for NormalMap {
    type Target = ImageData;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl DerefMut for NormalMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

impl TextureBinding for NormalMap {
    type State = ();

    fn bind_key(&self) -> BindKey {
        TextureBinding::bind_key(&self.0)
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        TextureBinding::binding(&self.0, device, queue, state)
    }
}

impl SamplerBinding for NormalMap {
    type State = Option<SharedSampler>;

    fn bind_key(&self) -> BindKey {
        TextureBinding::bind_key(&self.0)
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        SamplerBinding::binding(&self.0, device, queue, state)
    }
}

impl DefaultTexture for NormalMap {
    fn default_texture(device: &Device, queue: &Queue) -> SharedTextureView {
        let image =
            ImageData::with_format(1, 1, vec![127, 127, 255, 255], TextureFormat::Rgba8Unorm);
        image.create_view(device, queue)
    }
}

impl DefaultSampler for NormalMap {
    fn default_sampler(device: &Device, queue: &Queue) -> SharedSampler {
        Image::default_sampler(device, queue)
    }
}
