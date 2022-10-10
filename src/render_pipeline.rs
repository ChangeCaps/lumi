use std::{ops::Deref, sync::Arc};

use wgpu::RenderPipeline;

use crate::id::RenderPipelineId;

#[derive(Clone, Debug)]
pub struct SharedRenderPipeline {
    render_pipeline: Arc<RenderPipeline>,
    id: RenderPipelineId,
}

impl SharedRenderPipeline {
    pub fn new(render_pipeline: RenderPipeline) -> Self {
        Self {
            render_pipeline: Arc::new(render_pipeline),
            id: RenderPipelineId::new(),
        }
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }

    pub fn id(&self) -> RenderPipelineId {
        self.id
    }
}

impl Deref for SharedRenderPipeline {
    type Target = RenderPipeline;

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
