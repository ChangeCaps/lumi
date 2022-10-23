use lumi_util::{async_trait, thiserror};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[async_trait]
pub trait AssetIo: Send + Sync + 'static {
    async fn read(&self, path: &Path) -> Result<Vec<u8>, AssetIoError>;

    async fn exists(&self, path: &Path) -> bool;
}

impl dyn AssetIo {
    #[inline]
    pub fn read_blocking(&self, path: &Path) -> Result<Vec<u8>, AssetIoError> {
        lumi_task::block_on(self.read(path))
    }

    #[inline]
    pub async fn read_to_string(&self, path: &Path) -> Result<String, AssetIoError> {
        let bytes = self.read(path).await?;
        Ok(String::from_utf8(bytes)?)
    }

    #[inline]
    pub fn read_to_string_blocking(&self, path: &Path) -> Result<String, AssetIoError> {
        lumi_task::block_on(self.read_to_string(path))
    }

    #[inline]
    pub fn exists_blocking(&self, path: &Path) -> bool {
        lumi_task::block_on(self.exists(path))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssetIoError {
    #[error("invalid path: `{0}`")]
    InvalidPath(PathBuf),
    #[error("io error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("invalid utf-8")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Clone, Debug)]
pub struct FileAssetIo {
    root: PathBuf,
}

impl FileAssetIo {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

#[async_trait]
impl AssetIo for FileAssetIo {
    #[inline]
    async fn read(&self, path: &Path) -> Result<Vec<u8>, AssetIoError> {
        let path = self.root.join(path);
        Ok(fs::read(&path)?)
    }

    #[inline]
    async fn exists(&self, path: &Path) -> bool {
        let path = self.root.join(path);

        path.exists()
    }
}
