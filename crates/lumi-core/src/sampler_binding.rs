use crate::{BindKey, SamplerBinding, SharedBindingResource, SharedSampler};

impl<T: SamplerBinding> SamplerBinding for &T {
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

impl SamplerBinding for SharedSampler {
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
        SharedBindingResource::Sampler(self.clone())
    }
}
