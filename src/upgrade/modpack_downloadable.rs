use super::Downloadable;
use crate::{
    modpack::curseforge::{deser_manifest, read_manifest_file, structs::Manifest},
    HOME,
};
// use ferinth::Ferinth;
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
    DownloadError(#[from] super::DownloadError),
    #[error("{}", .0)]
    IOError(#[from] std::io::Error),
    #[error("{}", .0)]
    ZipError(#[from] zip::result::ZipError),
    #[error("{}", .0)]
    JSONError(#[from] serde_json::error::Error),
}
type Result<T> = std::result::Result<T, Error>;

pub async fn get_curseforge_manifest<TF, UF>(
    curseforge: Arc<Furse>,
    project_id: i32,
    total: TF,
    update: UF,
) -> Result<Manifest>
where
    TF: Fn(u64) + Send,
    UF: Fn(usize) + Send,
{
    let latest_file: Downloadable = curseforge
        .get_mod_files(project_id)
        .await?
        .swap_remove(0)
        .into();
    let cache_dir = HOME.join(".config").join("ferium").join(".cache");
    create_dir_all(&cache_dir).await?;
    let modpack_path = cache_dir.join(&latest_file.filename);
    if !modpack_path.exists() {
        latest_file.download(&cache_dir, total, update).await?;
    }
    Ok(deser_manifest(&read_manifest_file(File::open(
        modpack_path,
    )?)?)?)
}

// pub async fn get_modrinth_manifest(
//     modrinth: Arc<Ferinth>,
//     project_id: &str,
//     progress: F,
// ) -> Result<Manifest>
// where
//     F: Fn(usize, u64),
// {
//     ???
// }
