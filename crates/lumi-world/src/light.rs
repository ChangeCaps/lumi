use lumi_bounds::{CascadeFrustum, Frustum};
use lumi_macro::ShaderType;
use lumi_util::math::{Mat4, Vec3};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct RawPointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    /// The position of the light in world space.
    pub position: Vec3,
    /// The color of the light.
    pub color: Vec3,
    /// The intensity of the light in lumens.
    pub intensity: f32,
    /// The range of the light in meters.
    pub range: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            color: Vec3::ONE,
            intensity: 800.0,
            range: 20.0,
        }
    }
}

impl AsLight for PointLight {
    fn as_light(light: &Light) -> Option<&Self> {
        match light {
            Light::Point(point) => Some(point),
            _ => None,
        }
    }

    fn as_light_mut(light: &mut Light) -> Option<&mut Self> {
        match light {
            Light::Point(point) => Some(point),
            _ => None,
        }
    }

    fn from_light(light: Light) -> Option<Self> {
        match light {
            Light::Point(point) => Some(point),
            _ => None,
        }
    }
}

impl PointLight {
    pub fn raw(&self) -> RawPointLight {
        let intensity = self.intensity / (4.0 * std::f32::consts::PI);

        RawPointLight {
            position: self.position,
            color: self.color,
            intensity,
            range: self.range,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct RawDirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub size: f32,
    pub depth: f32,
    pub softness: f32,
    pub falloff: f32,
    pub cascade: u32,
    pub view_proj: Mat4,
}

#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight {
    /// Translation of the shadow map projection.
    ///
    /// This should usually be the position of the main camera.
    pub translation: Vec3,
    /// Direction of the light.
    pub direction: Vec3,
    /// Color of the light.
    pub color: Vec3,
    /// Intensity of the light in lux.
    pub illuminance: f32,
    /// Enables shadows.
    pub shadows: bool,
    /// Size of the shadow projection in meters.
    pub size: f32,
    /// The depth of the light frustum in meters.
    pub depth: f32,
    /// The softness of the shadows cast by this light.
    pub shadow_softness: f32,
    /// The falloff of the shadows cast by this light.
    pub shadow_falloff: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            illuminance: 100_000.0,
            shadows: true,
            size: 200.0,
            depth: 1000.0,
            shadow_softness: 2.0,
            shadow_falloff: 2.0,
        }
    }
}

impl AsLight for DirectionalLight {
    fn as_light(light: &Light) -> Option<&Self> {
        match light {
            Light::Directional(directional) => Some(directional),
            _ => None,
        }
    }

    fn as_light_mut(light: &mut Light) -> Option<&mut Self> {
        match light {
            Light::Directional(directional) => Some(directional),
            _ => None,
        }
    }

    fn from_light(light: Light) -> Option<Self> {
        match light {
            Light::Directional(directional) => Some(directional),
            _ => None,
        }
    }
}

impl DirectionalLight {
    pub const CASCADES: u32 = 4;

    pub fn view(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.translation);
        translation * Mat4::look_at_rh(-self.direction, Vec3::ZERO, Vec3::Y)
    }

    pub fn cascade_size(&self, cascade: u32) -> f32 {
        let size = 2.0_f32.powi(cascade as i32);
        let max = 2.0_f32.powi(4);
        self.size * size / max
    }

    pub fn proj(&self, cascade: u32) -> Mat4 {
        let size = self.cascade_size(cascade);

        let min = -size / 2.0;
        let max = size / 2.0;

        let near = -self.depth / 2.0;
        let far = self.depth / 2.0;

        Mat4::orthographic_rh(min, max, min, max, near, far)
    }

    pub fn cascade_frustum(&self, cascade: u32) -> CascadeFrustum {
        let view = self.view();
        let far = self.depth / 2.0;

        let outer = Frustum::from_view_proj(view, self.proj(cascade), far);
        let inner = if cascade > 0 {
            Some(Frustum::from_view_proj(view, self.proj(cascade - 1), far))
        } else {
            None
        };

        CascadeFrustum { outer, inner }
    }

    pub fn view_proj(&self, cascade: u32) -> Mat4 {
        self.proj(cascade) * self.view()
    }

    pub fn raw(&self, cascade: u32) -> RawDirectionalLight {
        RawDirectionalLight {
            direction: self.direction.normalize(),
            color: self.color,
            intensity: self.illuminance,
            size: self.size,
            depth: self.depth,
            softness: self.shadow_softness,
            falloff: self.shadow_falloff,
            cascade,
            view_proj: self.view_proj(0),
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct RawAmbientLight {
    pub color: Vec3,
}

impl Default for RawAmbientLight {
    fn default() -> Self {
        Self { color: Vec3::ONE }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AmbientLight {
    pub color: Vec3,
    pub intensity: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Vec3::ONE,
            intensity: 15000.0,
        }
    }
}

impl AmbientLight {
    pub fn raw(&self) -> RawAmbientLight {
        RawAmbientLight {
            color: self.color * self.intensity,
        }
    }
}

pub trait AsLight: Sized {
    fn as_light(light: &Light) -> Option<&Self>;
    fn as_light_mut(light: &mut Light) -> Option<&mut Self>;
    fn from_light(light: Light) -> Option<Self>;
}

#[derive(Clone, Copy, Debug)]
pub enum Light {
    Point(PointLight),
    Directional(DirectionalLight),
}

impl AsLight for Light {
    fn as_light(light: &Light) -> Option<&Self> {
        Some(light)
    }

    fn as_light_mut(light: &mut Light) -> Option<&mut Self> {
        Some(light)
    }

    fn from_light(light: Light) -> Option<Self> {
        Some(light)
    }
}

impl From<PointLight> for Light {
    fn from(light: PointLight) -> Self {
        Self::Point(light)
    }
}

impl From<DirectionalLight> for Light {
    fn from(light: DirectionalLight) -> Self {
        Self::Directional(light)
    }
}

impl Light {
    pub fn shadows(&self) -> bool {
        match self {
            Light::Point(_) => false,
            Light::Directional(directional) => directional.shadows,
        }
    }
}
