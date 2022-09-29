mod io;
mod processor;

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use naga::valid::{Capabilities, ValidationFlags, Validator};
use wgpu::{ShaderModule, ShaderSource};

use crate::Device;

pub use io::*;
pub use processor::*;

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

    pub fn language(&self) -> Result<ShaderLanguage, ShaderError> {
        match self {
            Self::Default(_) => Ok(ShaderLanguage::Wgsl),
            Self::Path(path) => ShaderLanguage::from_path(path),
            Self::Module(path) => ShaderLanguage::from_path(Path::new(path.as_ref())),
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

#[derive(Debug)]
pub struct Shader {
    module: naga::Module,
    wgsl: String,
    shader_module: Option<ShaderModule>,
}

impl Shader {
    fn rebind_modules(
        this: &mut naga::Module,
        other: &mut naga::Module,
    ) -> Result<(), ShaderError> {
        let mut bindings = HashMap::<Option<String>, u32>::new();

        for (_handle, variable) in this.global_variables.iter_mut() {
            if let Some(ref mut binding) = variable.binding {
                match bindings.get(&variable.name) {
                    Some(&rebinding) => {
                        binding.binding = rebinding;
                    }
                    None => {
                        binding.binding = bindings.len() as u32;

                        bindings.insert(variable.name.clone(), binding.binding);
                    }
                }
            }
        }

        for (_handle, variable) in other.global_variables.iter_mut() {
            if let Some(ref mut binding) = variable.binding {
                match bindings.get(&variable.name) {
                    Some(&rebinding) => {
                        binding.binding = rebinding;
                    }
                    None => {
                        binding.binding = bindings.len() as u32;

                        bindings.insert(variable.name.clone(), binding.binding);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn rebind(&mut self, other: &mut Self) -> Result<(), ShaderError> {
        Self::rebind_modules(&mut self.module, &mut other.module)?;

        Ok(())
    }

    pub fn new(source: &str, language: ShaderLanguage) -> Result<Self, ShaderError> {
        match language {
            ShaderLanguage::Wgsl => Self::from_wgsl(source),
            ShaderLanguage::Glsl => unimplemented!(),
        }
    }

    pub fn from_wgsl(wgsl: &str) -> Result<Self, ShaderError> {
        let module = naga::front::wgsl::parse_str(wgsl)?;

        Ok(Self {
            module,
            wgsl: String::new(),
            shader_module: None,
        })
    }

    pub fn compile(&mut self, device: &Device) -> Result<(), ShaderError> {
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
        let module_info = validator.validate(&self.module)?;

        self.wgsl = naga::back::wgsl::write_string(
            &self.module,
            &module_info,
            naga::back::wgsl::WriterFlags::empty(),
        )?
        .into();

        self.shader_module = Some(self.create_shader_module(device));

        Ok(())
    }

    pub fn module(&self) -> &naga::Module {
        &self.module
    }

    pub fn get_shader_module(&self) -> Option<&ShaderModule> {
        self.shader_module.as_ref()
    }

    pub fn create_shader_module(&self, device: &Device) -> ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(&self.wgsl)),
        })
    }

    pub fn shader_module(&mut self, device: &Device) -> &ShaderModule {
        if self.shader_module.is_none() {
            self.compile(device).unwrap();
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
