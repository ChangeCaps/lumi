use crate::{
    BindKey, SamplerBinding, SharedBindingResource, SharedDevice, SharedSampler, SharedTextureView,
    TextureBinding,
};

impl<T: TextureBinding> TextureBinding for &T {
    type State = T::State;

    #[inline]
    fn bind_key(&self) -> BindKey {
        T::bind_key(*self)
    }

    #[inline]
    fn binding(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        T::binding(*self, device, queue, state)
    }
}

impl TextureBinding for SharedTextureView {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(self.id())
    }

    #[inline]
    fn binding(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        SharedBindingResource::TextureView(self.clone())
    }
}

impl SamplerBinding for SharedTextureView {
    type State = Option<SharedSampler>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(self.id())
    }

    #[inline]
    fn binding(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        if let Some(sampler) = state {
            SharedBindingResource::Sampler(sampler.clone())
        } else {
            let sampler = device.create_shared_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
            *state = Some(sampler.clone());

            SharedBindingResource::Sampler(sampler)
        }
    }
}
