use std::path::{Path, PathBuf};
use tokio::fs;

use crate::file_system::{AsyncFileSystem, FsError};

pub struct LocalFs {
    root: PathBuf,
}

impl LocalFs {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[async_trait::async_trait]
impl AsyncFileSystem for LocalFs {
    fn root(&self) -> &Path {
        &self.root
    }

    async fn exists(&self, path: &Path) -> Result<bool, FsError> {
        Ok(fs::try_exists(path).await?)
    }

    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        Ok(fs::read(path).await?)
    }

    async fn try_read_file(&self, path: &Path) -> Result<Option<Vec<u8>>, FsError> {
        match fs::read(path).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), FsError> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).await?;
        }

        let path_buf = path.to_path_buf();
        let data_vec = data.to_vec();

        tokio::task::spawn_blocking(move || {
            let parent = path_buf
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let mut temp_file = tempfile::Builder::new()
                .prefix(".tmp")
                .tempfile_in(parent)?;
            use std::io::Write;
            temp_file.write_all(&data_vec)?;
            temp_file.as_file().sync_all()?;
            temp_file.persist(&path_buf).map_err(|e| e.error)?;
            Ok::<(), std::io::Error>(())
        })
        .await
        .unwrap()?;

        Ok(())
    }

    async fn delete_file(&self, path: &Path) -> Result<(), FsError> {
        fs::remove_file(path).await?;
        Ok(())
    }
}
