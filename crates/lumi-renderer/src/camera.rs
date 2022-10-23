use lumi_core::{Device, RenderTarget};
use lumi_world::Camera;

use crate::FrameBuffer;

pub struct PreparedCamera {
    pub frame_buffer: FrameBuffer,
}

impl PreparedCamera {
    pub fn new(
        device: &Device,
        camera: &Camera,
        target: &RenderTarget<'_>,
        sample_count: u32,
    ) -> Self {
        let width = camera.target.get_width(target);
        let height = camera.target.get_height(target);
        let frame_buffer = FrameBuffer::new(device, width, height, sample_count);
        Self { frame_buffer }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        camera: &Camera,
        target: &RenderTarget<'_>,
        sample_count: u32,
    ) {
        let width = camera.target.get_width(target);
        let height = camera.target.get_height(target);

        self.frame_buffer
            .resize(device, width, height, sample_count);
    }
}
