use ahash::HashMap;
use encase::ShaderType;
use glam::{Mat4, Vec3};

use crate::{
    aabb::{Frustum, RenderFrustum},
    bind::Bind,
    buffer::StorageBuffer,
    id::LightId,
};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct RawPointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
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
    pub translation: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub illuminance: f32,
    pub shadow_softness: f32,
    pub shadow_falloff: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            illuminance: 100_000.0,
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
    const NEAR: f32 = -500.0;
    const FAR: f32 = 500.0;
    const SIZE: f32 = 50.0;

    pub fn view(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.translation);
        translation * Mat4::look_at_rh(-self.direction, Vec3::ZERO, Vec3::Y)
    }

    pub fn proj(&self, cascade: u32) -> Mat4 {
        let cascade = cascade as f32;
        let size = f32::powf(2.0, cascade) * Self::SIZE / 2.0;

        Mat4::orthographic_rh(-size, size, -size, size, Self::NEAR, Self::FAR)
    }

    pub fn frustum(&self, cascade: u32) -> Frustum {
        let view = self.view();
        let proj = self.proj(cascade);

        Frustum::from_view_proj(view.inverse(), proj, Self::FAR)
    }

    pub fn render_frustum(&self, cascade: u32) -> RenderFrustum {
        let instersects = self.frustum(cascade);
        let without = if cascade > 0 {
            Some(self.frustum(cascade - 1))
        } else {
            None
        };

        RenderFrustum {
            intersects: instersects,
            without,
            intersect_far: true,
        }
    }

    pub fn view_proj(&self, cascade: u32) -> Mat4 {
        self.proj(cascade) * self.view()
    }

    pub fn raw(&self, cascade: u32) -> RawDirectionalLight {
        RawDirectionalLight {
            direction: self.direction.normalize(),
            color: self.color,
            intensity: self.illuminance,
            size: Self::SIZE,
            depth: Self::FAR - Self::NEAR,
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
            intensity: 35000.0,
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

#[derive(Default, Bind)]
pub struct LightBindings {
    #[uniform]
    pub ambient_light: RawAmbientLight,
    #[uniform]
    pub point_light_count: u32,
    #[storage_buffer]
    pub point_lights: StorageBuffer<RawPointLight>,
    #[uniform]
    pub directional_light_count: u32,
    pub directional_indices: HashMap<LightId, usize>,
    #[storage_buffer]
    pub directional_lights: StorageBuffer<RawDirectionalLight>,
}

impl LightBindings {
    pub fn clear(&mut self) {
        self.point_lights.clear();
        self.directional_indices.clear();
        self.directional_lights.clear();
    }

    pub fn push(&mut self, id: LightId, light: Light) {
        match light {
            Light::Point(point) => {
                self.point_lights.push(point.raw());
                self.point_light_count = self.point_lights.len() as u32;
            }
            Light::Directional(directional) => {
                self.directional_indices
                    .insert(id, self.directional_lights.len());

                self.directional_lights.push(directional.raw(0));
                self.directional_light_count = self.directional_lights.len() as u32;
            }
        }
    }
}
