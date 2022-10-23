use lumi_util::once_cell::sync::OnceCell;
use wgpu::{Device, Queue};

use crate::{
    BindKey, SamplerBinding, SharedBindingResource, SharedSampler, SharedTextureView,
    TextureBinding,
};

pub struct DefaultBindingState<T, U> {
    pub state: T,
    pub default_binding: OnceCell<U>,
}

impl<T: Default, U> Default for DefaultBindingState<T, U> {
    fn default() -> Self {
        Self {
            state: T::default(),
            default_binding: OnceCell::new(),
        }
    }
}

pub trait DefaultTexture {
    fn default_texture(device: &Device, queue: &Queue) -> SharedTextureView;
}

pub trait DefaultSampler {
    fn default_sampler(device: &Device, queue: &Queue) -> SharedSampler;
}

impl<T: TextureBinding + DefaultTexture> TextureBinding for Option<T> {
    type State = DefaultBindingState<T::State, SharedTextureView>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        if let Some(texture) = self {
            texture.bind_key()
        } else {
            BindKey::ZERO
        }
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        match self {
            Some(texture) => texture.binding(device, queue, &mut state.state),
            None => {
                let view = state
                    .default_binding
                    .get_or_init(|| T::default_texture(device, queue));

                SharedBindingResource::TextureView(view.clone())
            }
        }
    }
}

impl<T: SamplerBinding + DefaultSampler> SamplerBinding for Option<T> {
    type State = DefaultBindingState<T::State, SharedSampler>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        if let Some(sampler) = self {
            sampler.bind_key()
        } else {
            BindKey::ZERO
        }
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        match self {
            Some(texture) => texture.binding(device, queue, &mut state.state),
            None => {
                let sampler = state
                    .default_binding
                    .get_or_init(|| T::default_sampler(device, queue));

                SharedBindingResource::Sampler(sampler.clone())
            }
        }
    }
}
