use super::{check, DistributionDeniedError, Downloadable};
use crate::config::structs::{Mod, ModIdentifier, ModLoader};
use ferinth::{
    structures::version::{Version, VersionFile},
    Ferinth,
};
use furse::{structures::file_structs::File, Furse};
use octocrab::{
    models::repos::{Asset, Release},
    Octocrab,
};

#[derive(Debug, thiserror::Error)]
#[error("{}", .0)]
pub enum Error {
    DistributionDenied(#[from] DistributionDeniedError),
    ModrinthError(#[from] ferinth::Error),
    CurseForgeError(#[from] furse::Error),
    GitHubError(#[from] octocrab::Error),
    #[error("No compatible file was found")]
    NoCompatibleFile,
}
type Result<T> = std::result::Result<T, Error>;

/// Get the latest compatible version and version file from the `versions`
///
/// Also returns whether Fabric backwards compatibility for Quilt was used.
pub fn get_latest_compatible_version<'a>(
    versions: &'a [Version],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<(&'a VersionFile, &'a Version, bool)> {
    match check::modrinth(versions, game_version_to_check, mod_loader_to_check) {
        Some(some) => Some((some.0, some.1, false)),
        None => {
            if mod_loader_to_check == Some(&ModLoader::Quilt) {
                check::modrinth(versions, game_version_to_check, Some(&ModLoader::Fabric))
                    .map(|some| (some.0, some.1, true))
            } else {
                None
            }
        }
    }
}

/// Get the latest compatible file from the `files`
///
/// Also returns whether Fabric backwards compatibility for Quilt was used.
pub fn get_latest_compatible_file(
    mut files: Vec<File>,
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<(File, bool)> {
    match check::curseforge(&mut files, game_version_to_check, mod_loader_to_check) {
        Some(some) => Some((some.clone(), false)),
        None => {
            if mod_loader_to_check == Some(&ModLoader::Quilt) {
                check::curseforge(&mut files, game_version_to_check, Some(&ModLoader::Fabric))
                    .map(|some| (some.clone(), true))
            } else {
                None
            }
        }
    }
}

/// Get the latest compatible asset of the provided `repo_handler`.
/// Also returns whether Fabric backwards compatibility was used
pub fn get_latest_compatible_asset(
    releases: &[Release],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<(Asset, bool)> {
    match check::github(releases, game_version_to_check, mod_loader_to_check) {
        Some(some) => Some((some.clone(), false)),
        None => {
            if mod_loader_to_check == Some(&ModLoader::Quilt) {
                check::github(releases, game_version_to_check, Some(&ModLoader::Fabric))
                    .map(|some| (some.clone(), true))
            } else {
                None
            }
        }
    }
}

/// Get the latest compatible downloadable from the `mod_` provided.
/// Also returns whether fabric backwards compatibility was used
pub async fn get_latest_compatible_downloadable(
    modrinth: &Ferinth,
    curseforge: &Furse,
    github: &Octocrab,
    mod_: &Mod,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
) -> Result<(Downloadable, bool)> {
    let game_version_to_check = if mod_.check_game_version == Some(false) {
        None
    } else {
        Some(game_version_to_check)
    };
    let mod_loader_to_check = if mod_.check_mod_loader == Some(false) {
        None
    } else {
        Some(mod_loader_to_check)
    };

    match &mod_.identifier {
        ModIdentifier::CurseForgeProject(project_id) => get_latest_compatible_file(
            curseforge.get_mod_files(*project_id).await?,
            game_version_to_check,
            mod_loader_to_check,
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.try_into()?, ok.1)),
        ),
        ModIdentifier::ModrinthProject(project_id) => get_latest_compatible_version(
            &modrinth.list_versions(project_id).await?,
            game_version_to_check,
            mod_loader_to_check,
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.clone().into(), ok.2)),
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
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.into(), ok.1)),
        ),
    }
}
