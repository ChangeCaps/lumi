use naga::valid::{Capabilities, ValidationFlags, Validator};
use wgpu::ShaderModule;

use crate::SharedDevice;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShaderRef {
    Default,
    Path(&'static str),
}

pub struct Shader {
    module: naga::Module,
    spirv: Box<[u32]>,
    shader_module: Option<ShaderModule>,
}

impl Shader {
    pub fn from_wgsl(wgsl: &str) -> Result<Self, ShaderError> {
        let module = naga::front::wgsl::parse_str(wgsl)?;
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let module_info = validator.validate(&module)?;

        let spirv =
            naga::back::spv::write_vec(&module, &module_info, &Default::default(), None)?.into();

        Ok(Self {
            module,
            spirv,
            shader_module: None,
        })
    }

    pub fn module(&self) -> &naga::Module {
        &self.module
    }

    pub fn spirv(&self) -> &[u32] {
        &self.spirv
    }

    pub fn get_shader_module(&self) -> Option<&ShaderModule> {
        self.shader_module.as_ref()
    }

    pub fn create_shader_module(&self, device: &SharedDevice) -> ShaderModule {
        // SAFETY: we hope that naga produces valid spirv
        unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: None,
                source: self.spirv().into(),
            })
        }
    }

    pub fn shader_module(&mut self, device: &SharedDevice) -> &ShaderModule {
        if self.shader_module.is_none() {
            self.shader_module = Some(self.create_shader_module(device));
        }

        self.shader_module.as_ref().unwrap()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShaderError {
    #[error("validation error: {0}")]
    Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),
    #[error("parser error: {0}")]
    Wgsl(#[from] naga::front::wgsl::ParseError),
    #[error("spv error: {0}")]
    Spirv(#[from] naga::back::spv::Error),
}
