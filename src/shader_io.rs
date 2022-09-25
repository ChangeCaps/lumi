use std::{fs, path::Path};

pub type ShaderIoResult<T> = anyhow::Result<T>;
pub type ShaderIoError = anyhow::Error;

pub trait ShaderIo: Send + Sync {
    fn read(&self, path: &Path) -> ShaderIoResult<String>;
}

pub struct FsShaderIo;

impl ShaderIo for FsShaderIo {
    fn read(&self, path: &Path) -> ShaderIoResult<String> {
        Ok(fs::read_to_string(path)?)
    }
}

impl Default for Box<dyn ShaderIo> {
    fn default() -> Self {
        Box::new(FsShaderIo)
    }
}
