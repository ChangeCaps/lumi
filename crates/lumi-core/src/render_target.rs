use wgpu::TextureView;

pub struct RenderTarget<'a> {
    pub view: &'a TextureView,
    pub width: u32,
    pub height: u32,
}
