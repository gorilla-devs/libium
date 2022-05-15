pub mod check;
pub mod mod_downloadable;
pub mod modpack_downloadable;

use bytes::Bytes;
use ferinth::structures::version_structs::VersionFile;
use furse::{structures::file_structs::File, Furse};
use octocrab::models::repos::Asset;
use size::Size;
use std::{path::Path, sync::Arc};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("{}", .0)]
    ReqwestError(#[from] reqwest::Error),
    #[error("{}", .0)]
    IOError(#[from] std::io::Error),
}
type DownloadResult<T> = std::result::Result<T, DownloadError>;

#[derive(Debug, Clone)]
pub struct Downloadable {
    pub filename: String,
    pub download_url: String,
}
impl From<File> for Downloadable {
    fn from(file: File) -> Self {
        Self {
            filename: file.file_name,
            download_url: file.download_url,
        }
    }
}
impl From<VersionFile> for Downloadable {
    fn from(file: VersionFile) -> Self {
        Self {
            filename: file.filename,
            download_url: file.url,
        }
    }
}
impl From<Asset> for Downloadable {
    fn from(asset: Asset) -> Self {
        Self {
            filename: asset.name,
            download_url: asset.browser_download_url.into(),
        }
    }
}

impl Downloadable {
    /// Consumes `self` and downloads the file to the `output_dir`.
    ///
    /// Also provide a closure `update` to make a progress bar.
    /// The arguments provided are the amount the amount by which the data has increased, and the total amount of the data (zero if not known).
    ///
    /// Returns the size of the file and the filename
    pub async fn download<F>(
        self,
        output_dir: &Path,
        update: F,
    ) -> DownloadResult<(Size<usize>, String)>
    where
        F: Fn(usize, u64) + Send,
    {
        let mut contents = Bytes::new();
        let mut response = reqwest::get(&self.download_url).await?;
        while let Some(chunk) = response.chunk().await? {
            update(chunk.len(), response.content_length().unwrap_or(0));
            contents = [contents, chunk].concat().into();
        }
        let size = Size::Bytes(contents.len());
        let mut mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(output_dir.join(&self.filename))
            .await?;
        mod_file.write_all(&contents).await?;
        Ok((size, self.filename))
    }

    pub async fn from_ids(
        curseforge: Arc<Furse>,
        project_id: i32,
        file_id: i32,
    ) -> Result<Self, furse::Error> {
        let url = curseforge.file_download_url(project_id, file_id).await?;
        Ok(Self {
            filename: url
                .path_segments()
                .unwrap()
                .collect::<Vec<_>>()
                .iter()
                .last()
                .unwrap()
                .to_string(),
            download_url: url.into(),
        })
    }
}
