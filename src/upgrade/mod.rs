pub mod check;
pub mod mod_downloadable;
pub mod modpack_downloadable;

use ferinth::structures::version_structs::VersionFile;
use furse::{structures::file_structs::File, Furse};
use octocrab::models::repos::Asset;
use size::Size;
use std::{path::Path, sync::Arc};
use tokio::{
    fs::{rename, OpenOptions},
    io::AsyncWriteExt,
};
use urlencoding::decode;

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
    /// The `total` closure is called once if the content length is known.
    /// The `update` closure is called with the chunk length whenever a chunk is written
    ///
    /// Returns the size of the file and the filename
    pub async fn download<TF, UF>(
        self,
        output_dir: &Path,
        total: TF,
        update: UF,
    ) -> DownloadResult<(Size<usize>, String)>
    where
        TF: Fn(u64) + Send,
        UF: Fn(usize) + Send,
    {
        let mut response = reqwest::get(&self.download_url).await?;
        let out_file_path = output_dir.join(&self.filename).with_extension("part");
        let mut out_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(&out_file_path)
            .await?;
        if let Some(total_len) = response.content_length() {
            total(total_len);
        }
        let mut size = 0;
        while let Some(chunk) = response.chunk().await? {
            update(chunk.len());
            size += chunk.len();
            out_file.write_all(&chunk).await?;
        }
        rename(&out_file_path, output_dir.join(&self.filename)).await?;
        let size = Size::Bytes(size);
        Ok((size, self.filename))
    }

    /// Get a `Downloadable` from a project and file ID
    pub async fn from_ids(
        curseforge: Arc<Furse>,
        project_id: i32,
        file_id: i32,
    ) -> Result<Self, furse::Error> {
        let url = curseforge.file_download_url(project_id, file_id).await?;
        Ok(Self {
            filename: decode(
                url.path_segments()
                    .unwrap()
                    .collect::<Vec<_>>()
                    .iter()
                    .last()
                    .unwrap(),
            )
            .unwrap()
            .into(),
            download_url: url.into(),
        })
    }
}
