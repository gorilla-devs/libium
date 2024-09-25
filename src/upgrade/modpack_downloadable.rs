use super::{DistributionDeniedError, DownloadFile};
use crate::{CURSEFORGE_API, HOME, MODRINTH_API};
use reqwest::Client;
use std::{fs::create_dir_all, path::PathBuf};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    /// The user can manually download the modpack zip file and place it in `~/.config/ferium/.cache/` to mitigate this.
    /// However, they will have to manually update the modpack file.
    DistributionDenied(#[from] DistributionDeniedError),
    ModrinthError(#[from] ferinth::Error),
    CurseForgeError(#[from] furse::Error),
    ReqwestError(#[from] reqwest::Error),
    DownloadError(#[from] super::Error),
    IOError(#[from] std::io::Error),
    ZipError(#[from] zip::result::ZipError),
    JSONError(#[from] serde_json::error::Error),
}
type Result<T> = std::result::Result<T, Error>;

/// Download and open the latest file of `project_id`
///
/// Calls `total` once at the beginning with the file size if it is determined that the file needs to be downloaded.
/// Calls `update` with the chunk length whenever a chunk is downloaded and written.
pub async fn download_curseforge_modpack(
    project_id: i32,
    total: impl FnOnce(usize) + Send,
    update: impl Fn(usize) + Send,
) -> Result<PathBuf> {
    let latest_file: DownloadFile = CURSEFORGE_API
        .get_mod_files(project_id)
        .await?
        .swap_remove(0)
        .try_into()?;
    let cache_dir = HOME.join(".config").join("ferium").join(".cache");
    let modpack_path = cache_dir.join(&latest_file.output);
    if !modpack_path.exists() {
        create_dir_all(&cache_dir)?;
        total(latest_file.length);
        latest_file
            .download(&Client::new(), &cache_dir, update)
            .await?;
    }
    Ok(modpack_path)
}

/// Download and open the latest version of `project_id`
///
/// Calls `total` once at the beginning with the file size when it is determined that the file needs to be downloaded.
/// Calls `update` with the chunk length whenever a chunk is downloaded and written.
pub async fn download_modrinth_modpack(
    project_id: &str,
    total: impl FnOnce(usize) + Send,
    update: impl Fn(usize) + Send,
) -> Result<PathBuf> {
    let version_file: DownloadFile = MODRINTH_API
        .list_versions(project_id)
        .await?
        .swap_remove(0)
        .into();
    let cache_dir = HOME.join(".config").join("ferium").join(".cache");
    let modpack_path = cache_dir.join(&version_file.output);
    if !modpack_path.exists() {
        create_dir_all(&cache_dir)?;
        total(version_file.length);
        version_file
            .download(&Client::new(), &cache_dir, update)
            .await?;
    }
    Ok(modpack_path)
}
