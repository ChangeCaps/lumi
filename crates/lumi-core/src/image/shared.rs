use std::{
    num::NonZeroU8,
    ops::{Deref, DerefMut},
};

use lumi_util::{once_cell::sync::OnceCell, AtomicMarker, SharedState};
use wgpu::{
    AddressMode, BufferDescriptor, BufferUsages, FilterMode, ImageCopyBuffer, ImageCopyTexture,
    SamplerBorderColor, SamplerDescriptor, TextureFormat,
};

use crate::{
    BindKey, DefaultSampler, DefaultTexture, Device, Queue, SamplerBinding, SharedBindingResource,
    SharedDevice, SharedSampler, SharedTexture, SharedTextureView, StorageTextureBinding,
    TextureBinding, TextureId,
};

use super::ImageData;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageSamplerDescriptor {
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub anisotropy_clamp: Option<NonZeroU8>,
    pub border_color: Option<SamplerBorderColor>,
}

impl Default for ImageSamplerDescriptor {
    fn default() -> Self {
        Self {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            anisotropy_clamp: None,
            border_color: None,
        }
    }
}

impl From<&ImageSamplerDescriptor> for SamplerDescriptor<'_> {
    fn from(desc: &ImageSamplerDescriptor) -> Self {
        Self {
            label: None,
            address_mode_u: desc.address_mode_u,
            address_mode_v: desc.address_mode_v,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: desc.mag_filter,
            min_filter: desc.min_filter,
            mipmap_filter: desc.mipmap_filter,
            lod_min_clamp: desc.lod_min_clamp,
            lod_max_clamp: desc.lod_max_clamp,
            compare: None,
            anisotropy_clamp: desc.anisotropy_clamp,
            border_color: desc.border_color,
        }
    }
}

impl From<&SamplerDescriptor<'_>> for ImageSamplerDescriptor {
    fn from(desc: &SamplerDescriptor<'_>) -> Self {
        Self {
            address_mode_u: desc.address_mode_u,
            address_mode_v: desc.address_mode_v,
            mag_filter: desc.mag_filter,
            min_filter: desc.min_filter,
            mipmap_filter: desc.mipmap_filter,
            lod_min_clamp: desc.lod_min_clamp,
            lod_max_clamp: desc.lod_max_clamp,
            anisotropy_clamp: desc.anisotropy_clamp,
            border_color: desc.border_color,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Image {
    data: SharedState<ImageData>,
    write: AtomicMarker,
    texture: OnceCell<SharedTexture>,
    view: OnceCell<SharedTextureView>,
    pub sampler: ImageSamplerDescriptor,
}

impl Image {
    #[inline]
    pub fn new(image: ImageData) -> Self {
        Self {
            data: SharedState::new(image),
            ..Default::default()
        }
    }

    #[inline]
    pub fn new_render_target(width: u32, height: u32) -> Self {
        Self::new(ImageData::with_format(
            width,
            height,
            Vec::new(),
            TextureFormat::Bgra8UnormSrgb,
        ))
    }

    #[inline]
    #[cfg(feature = "image")]
    pub fn open_srgb(path: &str) -> Result<Self, image::ImageError> {
        let image = ImageData::open_srgb(path)?;
        Ok(Self::new(image))
    }

    #[inline]
    #[cfg(feature = "image")]
    pub fn open_rgb(path: &str) -> Result<Self, image::ImageError> {
        let image = ImageData::open_rgb(path)?;
        Ok(Self::new(image))
    }

    #[inline]
    #[cfg(feature = "image")]
    pub fn open_hdr(path: &str) -> Result<Self, image::ImageError> {
        let image = ImageData::open_hdr(path)?;
        Ok(Self::new(image))
    }

    #[inline]
    pub fn set_texture(&mut self, texture: SharedTexture) {
        if let Some(inner_texture) = self.texture.get_mut() {
            *inner_texture = texture;
        } else {
            self.texture
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
        self.texture.get()
    }

    #[inline]
    pub fn texture_eq(&self, other: &Self) -> bool {
        self.texture_id() == other.texture_id()
    }

    #[inline]
    pub fn data(&self) -> &ImageData {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut ImageData {
        if self.data.is_shared() {
            self.write.unmark();
            self.texture.take();
            self.view.take();
        } else {
            self.write.mark();
        }

        &mut self.data
    }

    #[inline]
    pub fn texture(&self, device: &Device, queue: &Queue) -> &SharedTexture {
        let should_write = self.write.take();

        let texture = self
            .texture
            .get_or_init(|| self.data.create_texture(device, queue));

        if should_write {
            queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Default::default(),
                    aspect: Default::default(),
                },
                &self.data.data,
                self.data.image_data_layout(),
                self.data.extent(),
            );
        }

        texture
    }

    #[inline]
    pub fn texture_view(&self, device: &Device, queue: &Queue) -> &SharedTextureView {
        let texture = self.texture(device, queue);

        self.view
            .get_or_init(|| texture.create_view(&Default::default()))
    }

    #[inline]
    pub fn download(&mut self, device: &Device, queue: &Queue) {
        let texture = if let Some(texture) = self.texture.get() {
            texture
        } else {
            return;
        };

        if self.write.is_marked() {
            return;
        }

        let stagning_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: self.data.data.len() as u64,
            usage: BufferUsages::COPY_SRC | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&Default::default());

        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            ImageCopyBuffer {
                buffer: &stagning_buffer,
                layout: self.data.image_data_layout_aligned(),
            },
            self.data.extent(),
        );

        let buffer_slice = stagning_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

        queue.submit(std::iter::once(encoder.finish()));

        let aligned_data = buffer_slice.get_mapped_range();
        self.data.copy_from_slice_aligned(&aligned_data);

        stagning_buffer.unmap();
    }
}

impl From<ImageData> for Image {
    #[inline]
    fn from(image: ImageData) -> Self {
        Self::new(image)
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
        BindKey::from_hash(self.texture_id())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        let view = self.texture_view(device, queue);
        SharedBindingResource::TextureView(view.clone())
    }
}

impl StorageTextureBinding for Image {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(self.texture_id())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        let view = self.texture_view(device, queue);
        SharedBindingResource::TextureView(view.clone())
    }
}

impl SamplerBinding for Image {
    type State = Option<SharedSampler>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(self.texture_id())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        _queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        let desc = SamplerDescriptor::from(&self.sampler);

        if let Some(sampler) = state {
            if sampler.descriptor() != desc {
                *sampler = device.create_shared_sampler(&desc);
            }

            SharedBindingResource::Sampler(sampler.clone())
        } else {
            let sampler = device.create_shared_sampler(&desc);

            *state = Some(sampler.clone());

            SharedBindingResource::Sampler(sampler)
        }
    }
}

impl DefaultTexture for Image {
    #[inline]
    fn default_texture(device: &Device, queue: &Queue) -> SharedTextureView {
        let data = ImageData::new(1, 1, vec![255; 4]);
        data.create_view(device, queue)
    }
}

impl DefaultSampler for Image {
    #[inline]
    fn default_sampler(device: &Device, _: &Queue) -> SharedSampler {
        device.create_shared_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
    }
}
