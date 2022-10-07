use encase::ShaderType;
use glam::{Mat4, Vec3};
use wgpu::TextureView;

use crate::{renderer::RenderTarget, SharedTextureView};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct RawCamera {
    pub position: Vec3,
    pub aspect_ratio: f32,
    pub view: Mat4,
    pub view_proj: Mat4,
    pub ev100: f32,
    pub exposure: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Perspective {
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: 45.0,
            aspect: 1.0,
            near: 0.1,
        }
    }
}

impl Perspective {
    pub fn new(fov: f32, aspect: f32, near: f32) -> Self {
        Self { fov, aspect, near }
    }

    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov.to_radians(), self.aspect, self.near)
    }

    pub fn projection_with_aspect(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_infinite_rh(self.fov.to_radians(), aspect, self.near)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orthographic {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Orthographic {
    fn default() -> Self {
        Self {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: -100.0,
            far: 100.0,
        }
    }
}

impl Orthographic {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    pub fn projection(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }

    pub fn projection_with_aspect(&self, aspect: f32) -> Mat4 {
        let width = self.right - self.left;
        let height = self.top - self.bottom;
        let new_width = width * aspect;
        let new_height = height * aspect;
        let left = self.left + (width - new_width) / 2.0;
        let right = self.right - (width - new_width) / 2.0;
        let bottom = self.bottom + (height - new_height) / 2.0;
        let top = self.top - (height - new_height) / 2.0;

        Mat4::orthographic_rh(left, right, bottom, top, self.near, self.far)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    Perspective(Perspective),
    Orthographic(Orthographic),
}

impl Default for Projection {
    fn default() -> Self {
        Self::Perspective(Perspective::default())
    }
}

impl Projection {
    pub fn projection(&self) -> Mat4 {
        match self {
            Projection::Perspective(perspective) => perspective.projection(),
            Projection::Orthographic(orthographic) => orthographic.projection(),
        }
    }

    pub fn projection_with_aspect(&self, aspect: f32) -> Mat4 {
        match self {
            Projection::Perspective(perspective) => perspective.projection_with_aspect(aspect),
            Projection::Orthographic(orthographic) => orthographic.projection_with_aspect(aspect),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CameraTarget {
    #[default]
    Main,
    Texture(SharedTextureView),
}

impl CameraTarget {
    pub fn get_view<'a>(&'a self, main: &'a TextureView) -> &'a TextureView {
        match self {
            CameraTarget::Main => main,
            CameraTarget::Texture(texture) => texture.view(),
        }
    }

    pub fn get_width(&self, main: u32) -> u32 {
        match self {
            CameraTarget::Main => main,
            CameraTarget::Texture(texture) => texture.size().width,
        }
    }

    pub fn get_height(&self, main: u32) -> u32 {
        match self {
            CameraTarget::Main => main,
            CameraTarget::Texture(texture) => texture.size().height,
        }
    }

    pub fn get_aspect(&self, main: &RenderTarget) -> f32 {
        let width = self.get_width(main.width) as f32;
        let height = self.get_height(main.height) as f32;
        width / height
    }
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub view: Mat4,
    pub projection: Projection,
    pub aperture: f32,
    pub shutter_speed: f32,
    pub sensitivity: f32,
    pub exposure_compensation: f32,
    pub target: CameraTarget,
    pub priority: u32,
    pub enabled: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view: Mat4::IDENTITY,
            projection: Projection::default(),
            target: CameraTarget::default(),
            aperture: 16.0,
            shutter_speed: 1.0 / 250.0,
            sensitivity: 100.0,
            exposure_compensation: 0.0,
            priority: 0,
            enabled: true,
        }
    }
}

impl Camera {
    pub fn with_view(mut self, view: Mat4) -> Self {
        self.view = view;
        self
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.view = Mat4::from_translation(position);
        self
    }

    pub fn with_projection(mut self, projection: Projection) -> Self {
        self.projection = projection;
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_target(mut self, target: CameraTarget) -> Self {
        self.target = target;
        self
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection.projection() * self.view.inverse()
    }

    pub fn view_proj_with_aspect(&self, aspect: f32) -> Mat4 {
        self.projection.projection_with_aspect(aspect) * self.view.inverse()
    }

    pub fn ev100(&self) -> f32 {
        let sensitivity = self.sensitivity / 100.0;
        let ev100 = f32::log2(self.aperture * self.aperture / self.shutter_speed * sensitivity);

        ev100 - self.exposure_compensation
    }

    pub fn exposure(&self) -> f32 {
        1.0 / f32::powf(2.0, self.ev100()) * 1.2
    }

    pub fn raw(&self) -> RawCamera {
        RawCamera {
            position: self.view.w_axis.truncate(),
            aspect_ratio: 1.0,
            view: self.view,
            view_proj: self.view_proj(),
            ev100: self.ev100(),
            exposure: self.exposure(),
        }
    }

    pub fn raw_with_aspect(&self, aspect: f32) -> RawCamera {
        RawCamera {
            position: self.view.w_axis.truncate(),
            aspect_ratio: aspect,
            view: self.view,
            view_proj: self.view_proj_with_aspect(aspect),
            ev100: self.ev100(),
            exposure: self.exposure(),
        }
    }

    pub fn info(&self, main: &RenderTarget) -> CameraInfo {
        CameraInfo {
            view: self.view,
            projection: self.projection,
            ev100: self.ev100(),
            exposure: self.exposure(),
            width: self.target.get_width(main.width),
            height: self.target.get_height(main.height),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraInfo {
    pub view: Mat4,
    pub projection: Projection,
    pub ev100: f32,
    pub exposure: f32,
    pub width: u32,
    pub height: u32,
}

impl CameraInfo {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn proj(&self) -> Mat4 {
        self.projection.projection_with_aspect(self.aspect_ratio())
    }

    pub fn view_proj(&self) -> Mat4 {
        self.proj() * self.view.inverse()
    }

    pub fn raw(&self) -> RawCamera {
        RawCamera {
            position: self.view.w_axis.truncate(),
            aspect_ratio: self.aspect_ratio(),
            view: self.view,
            view_proj: self.view_proj(),
            ev100: self.ev100,
            exposure: self.exposure,
        }
    }
}
