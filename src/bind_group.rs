use std::{ops::Deref, sync::Arc};

use wgpu::BindGroup;

use crate::id::BindGroupId;

#[derive(Clone, Debug)]
pub struct SharedBindGroup {
    bind_group: Arc<BindGroup>,
    id: BindGroupId,
}

impl SharedBindGroup {
    pub fn new(bind_group: BindGroup) -> Self {
        Self {
            bind_group: Arc::new(bind_group),
            id: BindGroupId::new(),
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn id(&self) -> BindGroupId {
        self.id
    }
}

impl Deref for SharedBindGroup {
    type Target = BindGroup;

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
