use super::{check, Downloadable};
use crate::config::structs::{Mod, ModIdentifier, ModLoader};
use ferinth::{
    structures::version_structs::{Version, VersionFile},
    Ferinth,
};
use furse::{structures::file_structs::File, Furse};
use octocrab::{
    models::repos::{Asset, Release},
    Octocrab,
};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", .0)]
    ModrinthError(#[from] ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(#[from] furse::Error),
    #[error("GitHub: {}", .0)]
    GitHubError(#[from] octocrab::Error),
    #[error("No compatible file was found")]
    NoCompatibleFile,
}
type Result<T> = std::result::Result<T, Error>;

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
) -> Option<(Asset, bool)> {
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

/// Get the latest compatible downloadable from the `mod_` provided.
/// Also returns whether fabric backwards compatibility was used
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
                .repos(&full_name.0, &full_name.1)
                .releases()
                .list()
                .send()
                .await?
                .items,
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
