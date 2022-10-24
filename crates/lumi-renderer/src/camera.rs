use lumi_bind::Bind;
use lumi_core::{Device, RenderTarget, SharedBuffer, UniformBuffer};
use lumi_world::{Camera, RawCamera};

use crate::FrameBuffer;

#[derive(Bind)]
pub struct PreparedCamera {
    #[uniform]
    pub camera: UniformBuffer<RawCamera>,
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
        let aspect = width as f32 / height as f32;
        let frame_buffer = FrameBuffer::new(device, width, height, sample_count);
        Self {
            camera: UniformBuffer::new(camera.raw_with_aspect(aspect)),
            frame_buffer,
        }
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

        let aspect = width as f32 / height as f32;
        self.camera.set(camera.raw_with_aspect(aspect));

        self.frame_buffer
            .resize(device, width, height, sample_count);
    }
}

#[derive(Bind)]
pub struct CameraBindings {
    #[uniform]
    pub camera: SharedBuffer,
}
