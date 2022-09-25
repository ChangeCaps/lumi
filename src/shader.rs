use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use naga::valid::{Capabilities, ValidationFlags, Validator};
use wgpu::{ShaderModule, ShaderSource};

use crate::{ShaderIoError, SharedDevice};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DefaultShader {
    Vertex,
    Fragment,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShaderRef {
    Default(DefaultShader),
    Path(Cow<'static, Path>),
    Module(Cow<'static, str>),
}

impl ShaderRef {
    pub fn new(path: &'static str) -> Self {
        Self::Path(Path::new(path).into())
    }

    pub fn module(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Module(name.into())
    }

    pub fn joined(self, parent_path: &Path) -> Self {
        match self {
            Self::Path(path) => Self::Path(parent_path.join(path).into()),
            _ => self,
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        match self {
            Self::Default(_) => None,
            Self::Path(path) => Some(PathBuf::from(path.as_ref())),
            Self::Module(_) => None,
        }
    }

    pub fn parent_path(&self) -> Option<PathBuf> {
        Some(self.path()?.parent()?.to_path_buf())
    }
}

pub struct Shader {
    module: naga::Module,
    wgsl: String,
    shader_module: Option<ShaderModule>,
}

impl Shader {
    fn match_bindings(this: &mut naga::Module, other: &naga::Module) -> Result<(), ShaderError> {
        for (_, variable) in other.global_variables.iter() {
            if let Some(binding) = variable.binding.clone() {
                let handle = this.global_variables.iter().find_map(|(handle, v)| {
                    if v.name == variable.name {
                        Some(handle)
                    } else {
                        None
                    }
                });

                if this
                    .global_variables
                    .iter()
                    .any(|(_, v)| v.binding != Some(binding.clone()) && v.name == variable.name)
                {
                    return Err(ShaderError::BindingMismatch(variable.name.clone()));
                }

                if let Some(handle) = handle {
                    let this_variable = &mut this.global_variables[handle];
                    if let Some(ref mut this_binding) = this_variable.binding {
                        if this_binding.group == binding.group {
                            this_binding.binding = binding.binding;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn from_wgsl(wgsl: &str, match_bindings: Option<&Self>) -> Result<Self, ShaderError> {
        let mut module = naga::front::wgsl::parse_str(wgsl)?;

        if let Some(other) = match_bindings {
            Self::match_bindings(&mut module, &other.module)?;
        }

        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
        let module_info = validator.validate(&module)?;

        let wgsl = naga::back::wgsl::write_string(
            &module,
            &module_info,
            naga::back::wgsl::WriterFlags::empty(),
        )?
        .into();

        Ok(Self {
            module,
            wgsl,
            shader_module: None,
        })
    }

    pub fn module(&self) -> &naga::Module {
        &self.module
    }

    pub fn get_shader_module(&self) -> Option<&ShaderModule> {
        self.shader_module.as_ref()
    }

    pub fn create_shader_module(&self, device: &SharedDevice) -> ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(&self.wgsl)),
        })
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
    #[error("wgsl error: {0}")]
    WgslOut(#[from] naga::back::wgsl::Error),
    #[error("binding mismatch: {0:?}")]
    BindingMismatch(Option<String>),
    #[error("No extension found for path {0:?}")]
    NoExtension(PathBuf),
    #[error("Unknown shader extension {0:?}")]
    UnknownExtension(String),
    #[error("Invalid include {0:?}")]
    InvalidInclude(String),
    #[error("Unclosed comment")]
    UnclosedComment,
    #[error("Invalid module {0:?}")]
    InvalidModule(String),
    #[error("Circular include {0:?}")]
    CircularInclude(ShaderRef),
    #[error("IO error: {0}")]
    IoError(#[from] ShaderIoError),
}
