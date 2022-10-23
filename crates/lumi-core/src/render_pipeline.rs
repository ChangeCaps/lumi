use std::{ops::Deref, sync::Arc};

use lumi_id::Id;

#[derive(Clone, Debug)]
pub struct SharedRenderPipeline {
    render_pipeline: Arc<wgpu::RenderPipeline>,
    id: Id<wgpu::RenderPipeline>,
}

impl SharedRenderPipeline {
    pub fn new(render_pipeline: wgpu::RenderPipeline) -> Self {
        Self {
            render_pipeline: Arc::new(render_pipeline),
            id: Id::new(),
        }
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    pub fn id(&self) -> Id<wgpu::RenderPipeline> {
        self.id
    }
}

impl Deref for SharedRenderPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.render_pipeline
    }
}

impl PartialEq for SharedRenderPipeline {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SharedRenderPipeline {}
