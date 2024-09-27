pub mod check;
pub mod mod_downloadable;
pub mod modpack_downloadable;

use crate::{
    config::{filters::ReleaseChannel, structs::ModLoader},
    iter_ext::IterExt as _,
    modpack::modrinth::structs::ModpackFile as ModpackModFile,
    version_ext::VersionExt,
};
use ferinth::structures::version::{Version as MRVersion, VersionType};
use furse::structures::file_structs::{File as CFFile, FileReleaseType};
use octocrab::models::repos::Asset as GHAsset;
use reqwest::{Client, Url};
use std::{
    fs::{create_dir_all, rename, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    ReqwestError(#[from] reqwest::Error),
    IOError(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct DownloadFile {
    pub game_versions: Vec<String>,
    pub loaders: Vec<ModLoader>,
    pub channel: ReleaseChannel,
    /// A URL to download the file from
    pub download_url: Url,
    /// The path of the file relative to the output directory
    ///
    /// Is just the filename by default, can be configured with subdirectories for modpacks.
    pub output: PathBuf,
    /// The length of the file in bytes
    pub length: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("The developer of this project has denied third party applications from downloading it")]
/// Contains the mod ID and file ID
pub struct DistributionDeniedError(pub i32, pub i32);

impl TryFrom<CFFile> for DownloadFile {
    type Error = DistributionDeniedError;
    fn try_from(file: CFFile) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            loaders: file
                .game_versions
                .iter()
                .filter_map(|s| ModLoader::from_str(s).ok())
                .collect_vec(),
            channel: match file.release_type {
                FileReleaseType::Release => ReleaseChannel::Release,
                FileReleaseType::Beta => ReleaseChannel::Beta,
                FileReleaseType::Alpha => ReleaseChannel::Alpha,
            },
            download_url: file
                .download_url
                .ok_or(DistributionDeniedError(file.mod_id, file.id))?,
            output: file.file_name.into(),
            length: file.file_length,
            game_versions: file.game_versions,
        })
    }
}

impl From<MRVersion> for DownloadFile {
    fn from(version: MRVersion) -> Self {
        Self {
            loaders: version
                .loaders
                .iter()
                .filter_map(|s| ModLoader::from_str(s).ok())
                .collect_vec(),
            channel: match version.version_type {
                VersionType::Release => ReleaseChannel::Release,
                VersionType::Beta => ReleaseChannel::Beta,
                VersionType::Alpha => ReleaseChannel::Alpha,
            },
            download_url: version.get_version_file().url.clone(),
            output: version.get_version_file().filename.as_str().into(),
            length: version.get_version_file().size,
            game_versions: version.game_versions,
        }
    }
}

impl From<ModpackModFile> for DownloadFile {
    fn from(file: ModpackModFile) -> Self {
        Self {
            game_versions: vec![],
            loaders: vec![],
            download_url: file
                .downloads
                .first()
                .expect("Download URLs not provided")
                .clone(),
            channel: ReleaseChannel::Release,
            output: file.path,
            length: file.file_size,
        }
    }
}

impl From<GHAsset> for DownloadFile {
    fn from(asset: GHAsset) -> Self {
        Self {
            game_versions: asset
                .name
                .strip_suffix(".jar")
                .unwrap_or("")
                .split('-')
                .map(ToOwned::to_owned)
                .collect_vec(),
            loaders: asset
                .name
                .strip_suffix(".jar")
                .unwrap_or("")
                .split('-')
                .filter_map(|s| ModLoader::from_str(s).ok())
                .collect_vec(),

            channel: ReleaseChannel::Release,

            download_url: asset.browser_download_url,
            output: PathBuf::from("mods").join(asset.name),
            length: asset.size as usize,
        }
    }
}

impl DownloadFile {
    /// Consumes `self` and downloads the file to the `output_dir`.
    ///
    /// The `update` closure is called with the chunk length whenever a chunk is downloaded and written.
    ///
    /// Returns the size of the file and the filename
    pub async fn download(
        self,
        client: &Client,
        output_dir: &Path,
        update: impl Fn(usize) + Send,
    ) -> Result<(usize, String)> {
        let (filename, url, size) = (self.filename(), self.download_url, self.length);
        let out_file_path = output_dir.join(&self.output);
        let temp_file_path = out_file_path.with_extension("part");
        if let Some(up_dir) = out_file_path.parent() {
            create_dir_all(up_dir)?;
        }

        let mut temp_file = BufWriter::with_capacity(
            size,
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(&temp_file_path)?,
        );

        let mut response = client.get(url).send().await?;

        while let Some(chunk) = response.chunk().await? {
            temp_file.write_all(&chunk)?;
            update(chunk.len());
        }
        temp_file.flush()?;
        rename(temp_file_path, out_file_path)?;
        Ok((size, filename))
    }

    pub fn filename(&self) -> String {
        self.output
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}
