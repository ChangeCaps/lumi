use crate::{
    BindKey, SharedBindingResource, SharedTexture, SharedTextureView, StorageTextureBinding,
    TextureId,
};

impl<T: StorageTextureBinding> StorageTextureBinding for &T {
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

impl StorageTextureBinding for SharedTexture {
    type State = Option<(TextureId, SharedTextureView)>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(self.id())
    }

    #[inline]
    fn binding(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        if let Some((id, view)) = state {
            if *id != self.id() {
                *id = self.id();
                *view = self.create_view(&Default::default());
            }

            SharedBindingResource::TextureView(view.clone())
        } else {
            let view = self.create_view(&Default::default());
            *state = Some((self.id(), view.clone()));

            SharedBindingResource::TextureView(view)
        }
    }
}

impl StorageTextureBinding for SharedTextureView {
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
