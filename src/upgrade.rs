use crate::{
    check,
    config::structs::{Mod, ModIdentifier, ModLoader},
};
use ferinth::{
    structures::version_structs::{Version, VersionFile},
    Ferinth,
};
use furse::{structures::file_structs::File, Furse};
use octocrab::{models::repos::Asset, repos::RepoHandler, Octocrab};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", .0)]
    ModrinthError(#[from] ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(#[from] furse::Error),
    #[error("{}", .0)]
    GitHubError(#[from] octocrab::Error),
    #[error("No compatible file was found")]
    NoCompatibleFile,
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Downloadable {
    pub filename: String,
    pub download_url: String,
}
impl From<furse::structures::file_structs::File> for Downloadable {
    fn from(file: furse::structures::file_structs::File) -> Self {
        Self {
            filename: file.file_name,
            download_url: file.download_url,
        }
    }
}
impl From<ferinth::structures::version_structs::VersionFile> for Downloadable {
    fn from(file: ferinth::structures::version_structs::VersionFile) -> Self {
        Self {
            filename: file.filename,
            download_url: file.url,
        }
    }
}
impl From<octocrab::models::repos::Asset> for Downloadable {
    fn from(asset: octocrab::models::repos::Asset) -> Self {
        Self {
            filename: asset.name,
            download_url: asset.browser_download_url.into(),
        }
    }
}

/// Get the latest compatible version and version file of the provided `project_id`.
/// Also returns whether Fabric backwards compatibility was used
pub async fn get_latest_compatible_version(
    modrinth: Arc<Ferinth>,
    project_id: &str,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(VersionFile, Version, bool)> {
    let versions = modrinth.list_versions(project_id).await?;
    match check::modrinth(
        &versions,
        game_version_to_check,
        mod_loader_to_check,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await
    {
        Some(some) => Ok((some.0.clone(), some.1.clone(), false)),
        None => {
            if mod_loader_to_check == &ModLoader::Quilt {
                check::modrinth(
                    &versions,
                    game_version_to_check,
                    &ModLoader::Fabric,
                    should_check_game_version,
                    should_check_mod_loader,
                )
                .await
                .map_or_else(
                    || Err(Error::NoCompatibleFile),
                    |some| Ok((some.0.clone(), some.1.clone(), true)),
                )
            } else {
                Err(Error::NoCompatibleFile)
            }
        },
    }
}

/// Get the latest compatible file of the provided `project_id`.
/// Also returns whether Fabric backwards compatibility was used
pub async fn get_latest_compatible_file(
    curseforge: Arc<Furse>,
    project_id: i32,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(File, bool)> {
    let mut files = curseforge.get_mod_files(project_id).await?;
    match check::curseforge(
        &mut files,
        game_version_to_check,
        mod_loader_to_check,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await
    {
        Some(some) => Ok((some.clone().into(), false)),
        None => {
            if mod_loader_to_check == &ModLoader::Quilt {
                check::curseforge(
                    &mut files,
                    game_version_to_check,
                    &ModLoader::Fabric,
                    should_check_game_version,
                    should_check_mod_loader,
                )
                .await
                .map_or_else(
                    || Err(Error::NoCompatibleFile),
                    |some| Ok((some.clone().into(), true)),
                )
            } else {
                Err(Error::NoCompatibleFile)
            }
        },
    }
}

/// Get the latest compatible asset of the provided `repo_handler`.
/// Also returns whether Fabric backwards compatibility was used
pub async fn get_latest_compatible_asset(
    repo_handler: RepoHandler<'_>,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(Asset, bool)> {
    let releases = repo_handler.releases().list().send().await?.items;
    match check::github(
        &releases,
        game_version_to_check,
        mod_loader_to_check,
        should_check_game_version,
        should_check_mod_loader,
    )
    .await
    {
        Some(some) => Ok((some.clone().into(), false)),
        None => {
            if mod_loader_to_check == &ModLoader::Quilt {
                check::github(
                    &releases,
                    game_version_to_check,
                    &ModLoader::Fabric,
                    should_check_game_version,
                    should_check_mod_loader,
                )
                .await
                .map_or_else(
                    || Err(Error::NoCompatibleFile),
                    |some| Ok((some.clone().into(), true)),
                )
            } else {
                Err(Error::NoCompatibleFile)
            }
        },
    }
}

pub async fn get_latest_compatible_downloadable(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    github: Arc<Octocrab>,
    mod_: &Mod,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
) -> Result<(Downloadable, bool)> {
    match &mod_.identifier {
        ModIdentifier::CurseForgeProject(project_id) => get_latest_compatible_file(
            curseforge,
            *project_id,
            game_version_to_check,
            mod_loader_to_check,
            mod_.check_game_version,
            mod_.check_mod_loader,
        )
        .await
        .map(|ok| (ok.0.into(), ok.1)),
        ModIdentifier::ModrinthProject(project_id) => get_latest_compatible_version(
            modrinth,
            project_id,
            game_version_to_check,
            mod_loader_to_check,
            mod_.check_game_version,
            mod_.check_mod_loader,
        )
        .await
        .map(|ok| (ok.0.into(), ok.2)),
        ModIdentifier::GitHubRepository(full_name) => get_latest_compatible_asset(
            github.repos(full_name.0.clone(), full_name.1.clone()),
            game_version_to_check,
            mod_loader_to_check,
            mod_.check_game_version,
            mod_.check_mod_loader,
        )
        .await
        .map(|ok| (ok.0.into(), ok.1)),
    }
}
