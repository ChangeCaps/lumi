use lumi_assets::{Asset, Handle};

use crate::{
    BindKey, DefaultSampler, DefaultTexture, SamplerBinding, SharedBindingResource, SharedSampler,
    SharedTextureView, StorageTextureBinding, TextureBinding,
};

impl<T: Asset + TextureBinding> TextureBinding for Handle<T> {
    type State = T::State;

    #[inline]
    fn bind_key(&self) -> BindKey {
        T::bind_key(self.wait())
    }

    #[inline]
    fn binding(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        T::binding(self.wait(), device, queue, state)
    }
}

impl<T: Asset + StorageTextureBinding> StorageTextureBinding for Handle<T> {
    type State = T::State;

    #[inline]
    fn bind_key(&self) -> BindKey {
        T::bind_key(self.wait())
    }

    #[inline]
    fn binding(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        T::binding(self.wait(), device, queue, state)
    }
}

impl<T: Asset + SamplerBinding> SamplerBinding for Handle<T> {
    type State = T::State;

    #[inline]
    fn bind_key(&self) -> BindKey {
        T::bind_key(self.wait())
    }

    #[inline]
    fn binding(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        T::binding(self.wait(), device, queue, state)
    }
}

impl<T: Asset + DefaultTexture> DefaultTexture for Handle<T> {
    #[inline]
    fn default_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> SharedTextureView {
        T::default_texture(device, queue)
    }
}

impl<T: Asset + DefaultSampler> DefaultSampler for Handle<T> {
    #[inline]
    fn default_sampler(device: &wgpu::Device, queue: &wgpu::Queue) -> SharedSampler {
        T::default_sampler(device, queue)
    }
}
