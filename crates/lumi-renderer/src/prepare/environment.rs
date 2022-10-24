use lumi_bind::Bind;
use lumi_core::{Device, Queue, Resources, SharedTextureView};
use lumi_world::{Environment, EnvironmentId, World};

use crate::{PhaseContext, RenderPhase};

#[derive(Clone, Debug, Bind)]
pub struct PreparedEnvironment {
    pub sky: SharedTextureView,
    #[texture(name = "environment_diffuse", dimension = cube)]
    #[sampler(name = "environment_sampler")]
    pub diffuse: SharedTextureView,
    #[texture(name = "environment_specular", dimension = cube)]
    pub specular: SharedTextureView,
    pub id: EnvironmentId,
}

impl PreparedEnvironment {
    pub fn new(device: &Device, queue: &Queue, environment: &Environment) -> Self {
        let baked = environment.bake(device, queue);

        Self {
            sky: baked.sky_view,
            diffuse: baked.irradiance_view,
            specular: baked.indirect_view,
            id: environment.id(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PrepareEnvironment;

impl RenderPhase for PrepareEnvironment {
    fn prepare(&mut self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        if let Some(prepared_environment) = resources.get::<PreparedEnvironment>() {
            if prepared_environment.id == world.environment().id() {
                return;
            }
        }

        let prepared = PreparedEnvironment::new(context.device, context.queue, world.environment());
        resources.insert(prepared);
    }
}
