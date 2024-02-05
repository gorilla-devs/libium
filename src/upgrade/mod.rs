pub mod check;
pub mod mod_downloadable;
pub mod modpack_downloadable;

use crate::modpack::modrinth::structs::ModpackFile;
use ferinth::structures::version::VersionFile;
use furse::structures::file_structs::File;
use octocrab::models::repos::Asset;
use reqwest::{Client, Url};
use std::path::{Path, PathBuf};
use tokio::{
    fs::{rename, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};

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
    /// The length of the file in bytes
    pub length: usize,
}

fn output_from_path(filename: &str) -> PathBuf {
    PathBuf::from(
        if filename
            .ends_with(".zip") { "resourcepacks" } else { "mods" },
    )
    .join(filename)
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
            output: output_from_path(&file.file_name),
            length: file.file_length,
        })
    }
}

impl From<VersionFile> for Downloadable {
    fn from(file: VersionFile) -> Self {
        Self {
            download_url: file.url,
            output: output_from_path(&file.filename),
            length: file.size,
        }
    }
}
impl From<ModpackFile> for Downloadable {
    fn from(file: ModpackFile) -> Self {
        Self {
            download_url: file
                .downloads
                .first()
                .expect("Downlaod url doesn't exist")
                .clone(),
            output: file.path,
            length: file.file_size,
        }
    }
}
impl From<Asset> for Downloadable {
    fn from(asset: Asset) -> Self {
        Self {
            download_url: asset.browser_download_url,
            output: PathBuf::from("mods").join(asset.name),
            length: asset.size as usize,
        }
    }
}

impl Downloadable {
    /// Consumes `self` and downloads the file to the `output_dir`.
    ///
    /// The `update` closure is called with the chunk length whenever a chunk is downloaded and written.
    ///
    /// Returns the size of the file and the filename
    pub async fn download<UF>(
        self,
        client: &Client,
        output_dir: &Path,
        mut update: UF,
    ) -> Result<(usize, String)>
    where
        UF: FnMut(usize) + Send,
    {
        let (filename, url, size) = (self.filename(), self.download_url, self.length);
        let out_file_path = output_dir.join(&self.output);
        let temp_file_path = out_file_path.with_extension("part");

        let mut temp_file = BufWriter::with_capacity(
            size,
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(&temp_file_path)
                .await?,
        );

        let mut response = client.get(url).send().await?;

        while let Some(chunk) = response.chunk().await? {
            temp_file.write_all(&chunk).await?;
            update(chunk.len());
        }
        temp_file.shutdown().await?;
        rename(&temp_file_path, out_file_path).await?;
        Ok((size, filename))
    }

    pub fn filename(&self) -> String {
        self.output
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
