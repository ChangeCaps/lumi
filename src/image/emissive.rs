use std::ops::{Deref, DerefMut};

use crate::{
    DefaultSampler, DefaultTexture, Image, ImageData, SamplerBinding, SharedBindingResource,
    SharedDevice, SharedQueue, SharedSampler, SharedTextureView, TextureBinding,
};

#[derive(Clone, Debug, Default)]
pub struct EmissiveMap(Image);

impl EmissiveMap {
    pub fn new(image: Image) -> Self {
        Self(image)
    }

    pub fn load_from_file(path: &str) -> Result<Self, image::ImageError> {
        let image = Image::load_from_file(path)?;
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

impl From<ImageData> for EmissiveMap {
    fn from(image_data: ImageData) -> Self {
        Self::new(Image::new(image_data))
    }
}

impl From<Image> for EmissiveMap {
    fn from(image: Image) -> Self {
        Self::new(image)
    }
}

impl From<EmissiveMap> for Image {
    fn from(emissive_map: EmissiveMap) -> Self {
        emissive_map.0
    }
}

impl Deref for EmissiveMap {
    type Target = ImageData;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl DerefMut for EmissiveMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

impl TextureBinding for EmissiveMap {
    type State = ();

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        TextureBinding::binding(&self.0, device, queue, state)
    }
}

impl SamplerBinding for EmissiveMap {
    type State = Option<SharedSampler>;

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        SamplerBinding::binding(&self.0, device, queue, state)
    }
}

impl DefaultTexture for EmissiveMap {
    fn default_texture(device: &SharedDevice, queue: &SharedQueue) -> SharedTextureView {
        let image = ImageData::new(1, 1, vec![0; 4]);
        image.create_view(device, queue)
    }
}

impl DefaultSampler for EmissiveMap {
    fn default_sampler(device: &SharedDevice, queue: &SharedQueue) -> SharedSampler {
        Image::default_sampler(device, queue)
    }
}
