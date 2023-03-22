use super::{DistributionDeniedError, Downloadable};
use crate::{version_ext::VersionExt, HOME};
use ferinth::Ferinth;
use furse::Furse;
use reqwest::Client;
use tokio::fs::{create_dir_all, File};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "The developer of this modpack has denied third party applications from downloading it"
    )]
    /// The user can manually download the modpack zip file and place it in `~/.config/ferium/.cache/` to mitigate this.
    /// However, they will have to manually update the modpack file
    DistributionDenied(#[from] DistributionDeniedError),
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
    ZipError(#[from] async_zip::error::ZipError),
    #[error("{}", .0)]
    JSONError(#[from] serde_json::error::Error),
}
type Result<T> = std::result::Result<T, Error>;

/// Download and open the latest file of `project_id`
pub async fn download_curseforge_modpack<TF, UF>(
    curseforge: &Furse,
    project_id: i32,
    total: TF,
    update: UF,
) -> Result<File>
where
    TF: FnOnce(usize) + Send,
    UF: FnMut(usize) + Send,
{
    let latest_file = curseforge.get_mod_files(project_id).await?.swap_remove(0);
    let cache_dir = HOME.join(".config").join("ferium").join(".cache");
    let modpack_path = cache_dir.join(&latest_file.file_name);
    if !modpack_path.exists() {
        let mut latest_file: Downloadable = latest_file.try_into()?;
        latest_file.output = latest_file.filename().into();
        create_dir_all(&cache_dir).await?;
        total(latest_file.length);
        latest_file
            .download(&Client::new(), &cache_dir, update)
            .await?;
    }
    Ok(File::open(modpack_path).await?)
}

/// Download and open the latest version of `project_id`
pub async fn download_modrinth_modpack<TF, UF>(
    modrinth: &Ferinth,
    project_id: &str,
    total: TF,
    update: UF,
) -> Result<File>
where
    TF: Fn(usize) + Send,
    UF: Fn(usize) + Send,
{
    let mut version_file: Downloadable = modrinth
        .list_versions(project_id)
        .await?
        .swap_remove(0)
        .into_version_file()
        .into();
    version_file.output = version_file.filename().into();

    let cache_dir = HOME.join(".config").join("ferium").join(".cache");
    create_dir_all(&cache_dir).await?;
    let modpack_path = cache_dir.join(&version_file.output);
    if !modpack_path.exists() {
        total(version_file.length);
        version_file
            .download(&Client::new(), &cache_dir, update)
            .await?;
    }
    Ok(File::open(modpack_path).await?)
}
