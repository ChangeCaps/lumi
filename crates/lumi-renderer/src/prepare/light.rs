use lumi_bind::Bind;
use lumi_core::{StorageBuffer, UniformBuffer};
use lumi_util::HashMap;

use shiv::{
    query::Query,
    system::{Res, ResMut},
    world::Entity,
};
use shiv_transform::GlobalTransform;

use crate::{
    AmbientLight, DirectionalLight, Extract, PointLight, RawAmbientLight, RawDirectionalLight,
    RawPointLight,
};

#[derive(Default, Bind)]
pub struct PreparedLights {
    #[uniform]
    pub ambient_light: UniformBuffer<RawAmbientLight>,
    #[uniform]
    pub point_light_count: UniformBuffer<u32>,
    #[storage_buffer]
    pub point_lights: StorageBuffer<Vec<RawPointLight>>,
    #[uniform]
    pub directional_light_count: UniformBuffer<u32>,
    #[storage_buffer]
    pub directional_lights: StorageBuffer<Vec<RawDirectionalLight>>,
    pub next_cascade_index: u32,
    pub cascade_indices: HashMap<Entity, u32>,
    pub bindings_changed: bool,
}

impl PreparedLights {
    #[inline]
    pub fn clear(&mut self) {
        *self.ambient_light = RawAmbientLight::default();
        *self.point_light_count = 0;
        *self.directional_light_count = 0;

        self.point_lights.clear();
        self.directional_lights.clear();

        self.next_cascade_index = 0;
        self.cascade_indices.clear();
    }
}

pub fn extract_light_system(
    mut prepared_lights: ResMut<PreparedLights>,
    ambient_light: Option<Res<AmbientLight>>,
    point_lights: Extract<Query<(Entity, &PointLight, Option<&GlobalTransform>)>>,
    directional_lights: Extract<Query<(Entity, &DirectionalLight, Option<&GlobalTransform>)>>,
) {
    prepared_lights.clear();

    let point_light_cap = prepared_lights.point_lights.capacity();
    let directional_light_cap = prepared_lights.directional_lights.capacity();

    // prepare ambient light
    if let Some(ambient_light) = ambient_light {
        prepared_lights.ambient_light.set(ambient_light.raw());
    }

    // prepare point lights
    for (_, light, transform) in point_lights.iter() {
        let position = transform.map(|t| t.translation).unwrap_or_default();
        let raw_light = light.raw(position);

        prepared_lights.point_lights.push(raw_light);
        *prepared_lights.point_light_count += 1;
    }

    // prepare directional lights
    for (entity, light, transform) in directional_lights.iter() {
        let transform = transform.copied().unwrap_or_default();
        let mut light = light.clone();

        light.direction = transform.matrix.mul_vec3(light.direction);
        light.direction = light.direction.normalize_or_zero();

        let cascade = if light.shadows {
            let index = prepared_lights.next_cascade_index;

            prepared_lights.cascade_indices.insert(entity, index);
            prepared_lights.next_cascade_index += DirectionalLight::CASCADES as u32;

            index
        } else {
            0
        };

        let raw_light = light.raw(cascade);

        prepared_lights.directional_lights.push(raw_light);
        *prepared_lights.directional_light_count += 1;
    }

    let point = prepared_lights.point_lights.capacity() != point_light_cap;
    let directional = prepared_lights.directional_lights.capacity() != directional_light_cap;

    prepared_lights.bindings_changed = point || directional;
}
