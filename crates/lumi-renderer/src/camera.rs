use lumi_bounds::{CameraFrustum, Frustum};
use lumi_core::{RenderTarget, SharedTextureView, TextureView};
use lumi_macro::ShaderType;
use lumi_util::math::{Mat4, Vec3};
use shiv::{prelude::Bundle, world::Component};
use shiv_transform::{GlobalTransform, Transform};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct RawCamera {
    pub position: Vec3,
    pub aspect_ratio: f32,
    pub view: Mat4,
    pub inverse_view: Mat4,
    pub view_proj: Mat4,
    pub inverse_view_proj: Mat4,
    pub ev100: f32,
    pub exposure: f32,
}

/// A right-handed infinite perspective projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Perspective {
    /// The vertical field of view in radians.
    pub fov: f32,
    /// The aspect ratio of the camera.
    pub aspect: f32,
    /// The near plane of the camera.
    pub near: f32,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: 70.0,
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

/// A right-handed orthographic projection.
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

    pub fn has_far_plane(&self) -> bool {
        match self {
            Projection::Perspective(_) => false,
            Projection::Orthographic(_) => true,
        }
    }

    pub fn far(&self) -> Option<f32> {
        match self {
            Projection::Perspective(_) => None,
            Projection::Orthographic(orthographic) => Some(orthographic.far),
        }
    }
}

/// Target for a [`Camera`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CameraTarget {
    /// Render to main target.
    #[default]
    Main,
    /// Render to a texture.
    Texture(SharedTextureView),
}

impl CameraTarget {
    pub fn get_view<'a>(&'a self, main: &RenderTarget<'a>) -> &'a TextureView {
        match self {
            CameraTarget::Main => main.view,
            CameraTarget::Texture(texture) => texture.view(),
        }
    }

    pub fn get_width(&self, main: &RenderTarget) -> u32 {
        match self {
            CameraTarget::Main => main.width,
            CameraTarget::Texture(texture) => texture.size().width,
        }
    }

    pub fn get_height(&self, main: &RenderTarget) -> u32 {
        match self {
            CameraTarget::Main => main.height,
            CameraTarget::Texture(texture) => texture.size().height,
        }
    }

    pub fn get_aspect(&self, main: &RenderTarget) -> f32 {
        let width = self.get_width(main) as f32;
        let height = self.get_height(main) as f32;
        width / height
    }
}

#[derive(Component, Clone, Debug)]
pub struct Camera {
    pub projection: Projection,
    /// The cameras aperture in f-stops.
    pub aperture: f32,
    /// The cameras shutter speed in seconds.
    pub shutter_speed: f32,
    /// The cameras ISO.
    pub sensitivity: f32,
    pub exposure_compensation: f32,
    pub target: CameraTarget,
    pub msaa: bool,
    /// Priority for rendering this camera.
    ///
    /// Cameras with a higher priority will be rendered first.
    pub priority: u32,
    pub enabled: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Projection::default(),
            aperture: 16.0,
            shutter_speed: 1.0 / 250.0,
            sensitivity: 100.0,
            exposure_compensation: 0.0,
            target: CameraTarget::default(),
            msaa: true,
            priority: 0,
            enabled: true,
        }
    }
}

impl Camera {
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

    pub fn sample_count(&self) -> u32 {
        if self.msaa {
            4
        } else {
            1
        }
    }

    pub fn has_far_plane(&self) -> bool {
        self.projection.has_far_plane()
    }

    pub fn far(&self) -> Option<f32> {
        self.projection.far()
    }

    pub fn view_proj(&self, view: Mat4) -> Mat4 {
        self.projection.projection() * view.inverse()
    }

    pub fn view_proj_with_aspect(&self, view: Mat4, aspect: f32) -> Mat4 {
        self.projection.projection_with_aspect(aspect) * view.inverse()
    }

    pub fn frustum(&self, view: Mat4) -> Frustum {
        Frustum::from_view_proj(
            view,
            self.projection.projection(),
            self.far().unwrap_or(10_000.0),
        )
    }

    pub fn frustum_with_aspect(&self, view: Mat4, aspect: f32) -> Frustum {
        Frustum::from_view_proj(
            view,
            self.projection.projection_with_aspect(aspect),
            self.far().unwrap_or(10_000.0),
        )
    }

    pub fn camera_frustum(&self, view: Mat4, aspect: f32) -> CameraFrustum {
        CameraFrustum {
            frustum: self.frustum_with_aspect(view, aspect),
            view,
            proj: self.projection.projection_with_aspect(aspect),
            far: self.far(),
        }
    }

    pub fn ev100(&self) -> f32 {
        let sensitivity = self.sensitivity / 100.0;
        let ev100 = f32::log2(self.aperture * self.aperture / self.shutter_speed * sensitivity);

        ev100 - self.exposure_compensation
    }

    pub fn exposure(&self) -> f32 {
        1.0 / f32::powf(2.0, self.ev100()) * 1.2
    }

    pub fn raw(&self, view: Mat4) -> RawCamera {
        RawCamera {
            position: view.w_axis.truncate(),
            aspect_ratio: 1.0,
            view,
            inverse_view: view.inverse(),
            view_proj: self.view_proj(view),
            inverse_view_proj: self.view_proj(view).inverse(),
            ev100: self.ev100(),
            exposure: self.exposure(),
        }
    }

    pub fn raw_with_aspect(&self, view: Mat4, aspect: f32) -> RawCamera {
        RawCamera {
            position: view.w_axis.truncate(),
            aspect_ratio: aspect,
            view,
            inverse_view: view.inverse(),
            view_proj: self.view_proj_with_aspect(view, aspect),
            inverse_view_proj: self.view_proj_with_aspect(view, aspect).inverse(),
            ev100: self.ev100(),
            exposure: self.exposure(),
        }
    }
}

#[derive(Clone, Debug, Bundle)]
pub struct PerspectiveCameraBundle {
    pub camera: Camera,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for PerspectiveCameraBundle {
    #[inline]
    fn default() -> Self {
        Self {
            camera: Camera {
                projection: Projection::Perspective(Perspective {
                    fov: 70.0,
                    near: 0.1,
                    aspect: 1.0,
                }),
                ..Default::default()
            },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

#[derive(Clone, Debug, Bundle)]
pub struct OrthographicCameraBundle {
    pub camera: Camera,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for OrthographicCameraBundle {
    #[inline]
    fn default() -> Self {
        Self {
            camera: Camera {
                projection: Projection::Orthographic(Orthographic {
                    left: -5.0,
                    right: 5.0,
                    bottom: -5.0,
                    top: 5.0,
                    near: -100.0,
                    far: 100.0,
                }),
                ..Default::default()
            },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}
