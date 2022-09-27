use encase::ShaderType;
use glam::Vec3;

use crate::{bind::Bind, buffer::StorageBuffer};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct RawPointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub range: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            color: Vec3::ONE,
            intensity: 1.0,
            range: 20.0,
            radius: 0.0,
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
        RawPointLight {
            position: self.position,
            color: self.color * self.intensity,
            range: self.range,
            radius: self.radius,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct RawDirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 1.0,
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
    #[storage_buffer]
    pub point_lights: StorageBuffer<RawPointLight>,
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
            Light::Point(point) => self.point_lights.push(point.raw()),
            Light::Directional(directional) => self.directional_lights.push(directional.raw()),
        }
    }
}
