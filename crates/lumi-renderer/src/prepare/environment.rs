use lumi_bind::Bind;
use lumi_core::{Device, Queue, SharedTextureView};
use lumi_id::Id;
use shiv::system::{Commands, Res};

use crate::{Environment, EnvironmentId, Extract, RenderDevice, RenderQueue};

#[derive(Clone, Debug, Bind)]
pub struct PreparedEnvironment {
    pub sky: SharedTextureView,
    #[texture(name = "environment_diffuse", dimension = cube)]
    #[sampler(name = "environment_sampler")]
    pub irradiance: SharedTextureView,
    #[texture(name = "environment_specular", dimension = cube)]
    pub indirect: SharedTextureView,
    pub id: EnvironmentId,
}

impl PreparedEnvironment {
    pub fn new(device: &Device, queue: &Queue, environment: &Environment) -> Self {
        let baked = environment.bake(device, queue);

        Self {
            sky: baked.sky_view,
            irradiance: baked.irradiance_view,
            indirect: baked.indirect_view,
            id: environment.id(),
        }
    }
}

pub fn extract_environment_system(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    environment: Extract<Option<Res<Environment>>>,
    prepared_environment: Option<Res<PreparedEnvironment>>,
) {
    let prepared_id = prepared_environment.map(|env| env.id);

    if let Some(environment) = environment.as_deref() {
        if prepared_id != Some(environment.id()) {
            commands.insert_resource(PreparedEnvironment::new(&device, &queue, &environment));
        }
    } else if prepared_id != Some(Id::NULL) {
        let default = Environment::default();

        commands.insert_resource(PreparedEnvironment::new(&device, &queue, &default));
    }
}
