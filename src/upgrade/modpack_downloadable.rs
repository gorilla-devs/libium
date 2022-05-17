use super::Downloadable;
use crate::{version_ext::VersionExt, HOME};
use ferinth::Ferinth;
use furse::Furse;
use std::{fs::File, sync::Arc};
use tokio::fs::create_dir_all;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", .0)]
    ModrinthError(#[from] ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(#[from] furse::Error),
    #[error("{}", .0)]
    ReqwestError(#[from] reqwest::Error),
    #[error("{}", .0)]
    DownloadError(#[from] super::Error),
    #[error("{}", .0)]
    IOError(#[from] std::io::Error),
    #[error("{}", .0)]
    ZipError(#[from] zip::result::ZipError),
    #[error("{}", .0)]
    JSONError(#[from] serde_json::error::Error),
}
type Result<T> = std::result::Result<T, Error>;

pub async fn download_curseforge_modpack<TF, UF>(
    curseforge: Arc<Furse>,
    project_id: i32,
    total: TF,
    update: UF,
) -> Result<File>
where
    TF: Fn(u64) + Send,
    UF: Fn(usize) + Send,
{
    let latest_file = curseforge.get_mod_files(project_id).await?.swap_remove(0);
    let latest_file = Downloadable {
        download_url: latest_file.download_url,
        output: latest_file.file_name.into(),
        size: Some(latest_file.file_length),
    };
    let cache_dir = HOME.join(".config").join("ferium").join(".cache");
    create_dir_all(&cache_dir).await?;
    let modpack_path = cache_dir.join(&latest_file.output);
    if !modpack_path.exists() {
        latest_file.download(&cache_dir, total, update).await?;
    }
    Ok(File::open(modpack_path)?)
}

pub async fn download_modrinth_modpack<TF, UF>(
    modrinth: Arc<Ferinth>,
    project_id: &str,
    total: TF,
    update: UF,
) -> Result<File>
where
    TF: Fn(u64) + Send,
    UF: Fn(usize) + Send,
{
    let version_file = modrinth
        .list_versions(project_id)
        .await?
        .swap_remove(0)
        .into_version_file();
    let version_file = Downloadable {
        download_url: version_file.url,
        output: version_file.filename.into(),
        size: Some(version_file.size as u64),
    };

    let cache_dir = HOME.join(".config").join("ferium").join(".cache");
    create_dir_all(&cache_dir).await?;
    let modpack_path = cache_dir.join(&version_file.output);
    if !modpack_path.exists() {
        version_file.download(&cache_dir, total, update).await?;
    }
    Ok(File::open(modpack_path)?)
}