use encase::ShaderType;
use glam::Vec3;

use crate::{bind::Bind, buffer::StorageBuffer};

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
}

#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub illuminance: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            illuminance: 100_000.0,
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
    pub fn raw(&self) -> RawDirectionalLight {
        RawDirectionalLight {
            direction: self.direction.normalize(),
            color: self.color,
            intensity: self.illuminance,
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
    #[storage_buffer]
    pub directional_lights: StorageBuffer<RawDirectionalLight>,
}

impl LightBindings {
    pub fn clear(&mut self) {
        self.point_lights.clear();
        self.directional_lights.clear();
    }

    pub fn push(&mut self, light: Light) {
        match light {
            Light::Point(point) => {
                self.point_lights.push(point.raw());
                self.point_light_count = self.point_lights.len() as u32;
            }
            Light::Directional(directional) => {
                self.directional_lights.push(directional.raw());
                self.directional_light_count = self.directional_lights.len() as u32;
            }
        }
    }
}
