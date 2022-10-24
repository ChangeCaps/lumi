use lumi_bind::Bind;
use lumi_core::{Resources, StorageBuffer, UniformBuffer};
use lumi_util::HashMap;
use lumi_world::{
    DirectionalLight, Light, LightId, RawAmbientLight, RawDirectionalLight, RawPointLight, World,
};

use crate::{PhaseContext, RenderPhase};

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
    pub cascade_indices: HashMap<LightId, u32>,
}

impl PreparedLights {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

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

    #[inline]
    pub fn push(&mut self, id: LightId, light: &Light) {
        match light {
            Light::Point(point) => {
                self.point_lights.push(point.raw());
                *self.point_light_count += 1;
            }
            Light::Directional(directional) => {
                let cascade = self.next_cascade_index;

                self.directional_lights.push(directional.raw(cascade));
                *self.directional_light_count += 1;

                if directional.shadows {
                    self.cascade_indices.insert(id, cascade);
                    self.next_cascade_index += DirectionalLight::CASCADES;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PrepareLights;

impl RenderPhase for PrepareLights {
    fn prepare(&mut self, _: &PhaseContext, world: &World, resources: &mut Resources) {
        let prepared_lights = resources.get_or_default::<PreparedLights>();
        prepared_lights.clear();

        *prepared_lights.ambient_light = world.ambient().raw();

        for (id, point_light) in world.iter_lights() {
            prepared_lights.push(id, point_light);
        }
    }
}
