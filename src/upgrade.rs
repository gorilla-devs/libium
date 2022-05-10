use crate::{
    check,
    config::structs::{Mod, ModIdentifier, ModLoader},
};
use ferinth::{
    structures::version_structs::{Version, VersionFile},
    Ferinth,
};
use furse::{structures::file_structs::File, Furse};
use octorust::{
    types::{Release, ReleaseAsset},
    Client,
};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", .0)]
    ModrinthError(#[from] ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(#[from] furse::Error),
    #[error("GitHub: {}", .0)]
    GitHubError(#[from] anyhow::Error),
    #[error("No compatible file was found")]
    NoCompatibleFile,
}
type Result<T> = std::result::Result<T, Error>;

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
impl From<ReleaseAsset> for Downloadable {
    fn from(asset: ReleaseAsset) -> Self {
        Self {
            filename: asset.name,
            download_url: asset.browser_download_url,
        }
    }
}

/// Get the latest compatible version and version file of the provided `project_id`.
/// Also returns whether Fabric backwards compatibility was used
pub fn get_latest_compatible_version(
    versions: &[Version],
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Option<(VersionFile, Version, bool)> {
    match check::modrinth(
        versions,
        game_version_to_check,
        mod_loader_to_check,
        should_check_game_version,
        should_check_mod_loader,
    ) {
        Some(some) => Some((some.0.clone(), some.1.clone(), false)),
        None => {
            if mod_loader_to_check == &ModLoader::Quilt {
                check::modrinth(
                    versions,
                    game_version_to_check,
                    &ModLoader::Fabric,
                    should_check_game_version,
                    should_check_mod_loader,
                )
                .map(|some| (some.0.clone(), some.1.clone(), true))
            } else {
                None
            }
        },
    }
}

/// Get the latest compatible file of the provided `project_id`.
/// Also returns whether Fabric backwards compatibility was used
pub fn get_latest_compatible_file(
    mut files: Vec<File>,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Option<(File, bool)> {
    match check::curseforge(
        &mut files,
        game_version_to_check,
        mod_loader_to_check,
        should_check_game_version,
        should_check_mod_loader,
    ) {
        Some(some) => Some((some.clone(), false)),
        None => {
            if mod_loader_to_check == &ModLoader::Quilt {
                check::curseforge(
                    &mut files,
                    game_version_to_check,
                    &ModLoader::Fabric,
                    should_check_game_version,
                    should_check_mod_loader,
                )
                .map(|some| (some.clone(), true))
            } else {
                None
            }
        },
    }
}

/// Get the latest compatible asset of the provided `repo_handler`.
/// Also returns whether Fabric backwards compatibility was used
pub fn get_latest_compatible_asset(
    releases: &[Release],
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Option<(ReleaseAsset, bool)> {
    match check::github(
        releases,
        game_version_to_check,
        mod_loader_to_check,
        should_check_game_version,
        should_check_mod_loader,
    ) {
        Some(some) => Some((some.clone(), false)),
        None => {
            if mod_loader_to_check == &ModLoader::Quilt {
                check::github(
                    releases,
                    game_version_to_check,
                    &ModLoader::Fabric,
                    should_check_game_version,
                    should_check_mod_loader,
                )
                .map(|some| (some.clone(), true))
            } else {
                None
            }
        },
    }
}

pub async fn get_latest_compatible_downloadable(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    github: Arc<Client>,
    mod_: &Mod,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
) -> Result<(Downloadable, bool)> {
    match &mod_.identifier {
        ModIdentifier::CurseForgeProject(project_id) => get_latest_compatible_file(
            curseforge.get_mod_files(*project_id).await?,
            game_version_to_check,
            mod_loader_to_check,
            mod_.check_game_version,
            mod_.check_mod_loader,
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.into(), ok.1)),
        ),
        ModIdentifier::ModrinthProject(project_id) => get_latest_compatible_version(
            &modrinth.list_versions(project_id).await?,
            game_version_to_check,
            mod_loader_to_check,
            mod_.check_game_version,
            mod_.check_mod_loader,
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.into(), ok.2)),
        ),
        ModIdentifier::GitHubRepository(full_name) => get_latest_compatible_asset(
            &github
                .repos()
                .list_releases(&full_name.0, &full_name.1, 100, 0)
                .await?,
            game_version_to_check,
            mod_loader_to_check,
            mod_.check_game_version,
            mod_.check_mod_loader,
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.into(), ok.1)),
        ),
    }
}
