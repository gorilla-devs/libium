pub mod check;
pub mod mod_downloadable;
pub mod modpack_downloadable;

use crate::modpack::modrinth::structs::ModpackFile;
use ferinth::structures::version_structs::VersionFile;
use furse::{structures::file_structs::File, Furse};
use octocrab::models::repos::Asset;
use reqwest::Url;
use size::Size;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::{rename, OpenOptions},
    io::AsyncWriteExt,
};
use urlencoding::decode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", .0)]
    ReqwestError(#[from] reqwest::Error),
    #[error("{}", .0)]
    IOError(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Downloadable {
    /// A URL to download the file from
    pub download_url: Url,
    /// Where to output the file relative to the output directory (Minecraft instance directory)
    pub output: PathBuf,
    /// The size of the file in bytes
    pub size: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
#[error("The developer of this mod has denied third party applications from downloading it")]
pub struct DistributionDeniedError(pub i32, pub i32);
impl TryFrom<File> for Downloadable {
    type Error = DistributionDeniedError;
    fn try_from(file: File) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            download_url: file
                .download_url
                .ok_or(DistributionDeniedError(file.mod_id, file.id))?,
            output: PathBuf::from(if file.file_name.ends_with(".zip") {
                "resourcepacks"
            } else {
                "mods"
            })
            .join(file.file_name),
            size: Some(file.file_length as u64),
        })
    }
}
impl From<VersionFile> for Downloadable {
    fn from(file: VersionFile) -> Self {
        Self {
            download_url: file.url,
            output: PathBuf::from(if file.filename.ends_with(".zip") {
                "resourcepacks"
            } else {
                "mods"
            })
            .join(file.filename),
            size: Some(file.size as u64),
        }
    }
}
impl From<ModpackFile> for Downloadable {
    fn from(file: ModpackFile) -> Self {
        Self {
            download_url: file.downloads[0].clone(),
            output: file.path,
            size: Some(file.file_size),
        }
    }
}
impl From<Asset> for Downloadable {
    fn from(asset: Asset) -> Self {
        Self {
            download_url: asset.browser_download_url,
            output: PathBuf::from("mods").join(asset.name),
            size: Some(asset.size as u64),
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
    ) -> Result<(Option<Size>, String)>
    where
        TF: Fn(u64) + Send,
        UF: Fn(usize) + Send,
    {
        let mut file_size = None;
        if let Some(size) = self.size {
            total(size);
            file_size = Some(size);
        }
        let mut response = reqwest::get(self.download_url).await?;
        if let Some(size) = response.content_length() {
            if file_size.is_none() {
                total(size);
                file_size = Some(size);
            }
        }
        let out_file_path = output_dir.join(&self.output);
        let temp_file_path = out_file_path.with_extension("part");
        let mut temp_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(&temp_file_path)
            .await?;
        while let Some(chunk) = response.chunk().await? {
            update(chunk.len());
            temp_file.write_all(&chunk).await?;
        }
        rename(&temp_file_path, out_file_path).await?;
        Ok((
            file_size.map(Size::from_bytes),
            self.output
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ))
    }

    /// Get a `Downloadable` from a CurseForge project and file ID
    pub async fn from_file_id(
        curseforge: Arc<Furse>,
        project_id: i32,
        file_id: i32,
    ) -> std::result::Result<Self, furse::Error> {
        let url = curseforge.file_download_url(project_id, file_id).await?;
        let segments = url.path_segments().unwrap().collect::<Vec<_>>();
        let filename = decode(segments.iter().last().unwrap())
            .unwrap()
            .into_owned();
        Ok(Self {
            download_url: url,
            output: PathBuf::from(if filename.ends_with(".zip") {
                "resourcepacks"
            } else {
                "mods"
            })
            .join(filename),
            size: None,
        })
    }

    pub fn filename(&self) -> String {
        self.output
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
