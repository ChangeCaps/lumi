use std::{ops::Deref, sync::Arc};

use lumi_id::Id;

#[derive(Clone, Debug)]
pub struct SharedBindGroup {
    bind_group: Arc<wgpu::BindGroup>,
    id: Id<wgpu::BindGroup>,
}

impl SharedBindGroup {
    pub fn new(bind_group: wgpu::BindGroup) -> Self {
        Self {
            bind_group: Arc::new(bind_group),
            id: Id::new(),
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn id(&self) -> Id<wgpu::BindGroup> {
        self.id
    }
}

impl Deref for SharedBindGroup {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.bind_group
    }
}

impl PartialEq for SharedBindGroup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SharedBindGroup {}
