use super::{check, DistributionDeniedError, Downloadable};
use crate::{
    config::structs::{Mod, ModIdentifier, ModLoader},
    APIs,
};
use ferinth::structures::version::{Version, VersionFile};
use furse::structures::file_structs::File;
use octocrab::models::repos::{Asset, Release};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DistributionDenied(#[from] DistributionDeniedError),
    #[error("Modrinth: {0}")]
    ModrinthError(#[from] ferinth::Error),
    #[error("CurseForge: {0}")]
    CurseForgeError(#[from] furse::Error),
    #[error("GitHub: {0:#?}")]
    GitHubError(#[from] octocrab::Error),
    #[error("No compatible file was found")]
    NoCompatibleFile,
}
type Result<T> = std::result::Result<T, Error>;

/// Get the latest compatible version and version file from the `versions`
///
/// Also returns whether Fabric backwards compatibility for Quilt was used.
pub(crate) fn get_latest_compatible_version<'a>(
    versions: &'a [Version],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<ModLoader>,
) -> Option<(&'a VersionFile, &'a Version, bool)> {
    match check::modrinth(versions, game_version_to_check, mod_loader_to_check) {
        Some(some) => Some((some.0, some.1, false)),
        None => {
            if mod_loader_to_check == Some(ModLoader::Quilt) {
                check::modrinth(versions, game_version_to_check, Some(ModLoader::Fabric))
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
pub(crate) fn get_latest_compatible_file(
    mut files: Vec<File>,
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<ModLoader>,
) -> Option<(File, bool)> {
    match check::curseforge(&mut files, game_version_to_check, mod_loader_to_check) {
        Some(some) => Some((some.clone(), false)),
        None => {
            if mod_loader_to_check == Some(ModLoader::Quilt) {
                check::curseforge(&mut files, game_version_to_check, Some(ModLoader::Fabric))
                    .map(|some| (some.clone(), true))
            } else {
                None
            }
        }
    }
}

/// Get the latest compatible asset of the provided `repo_handler`
///
/// Also returns whether Fabric backwards compatibility for Quilt was used.
pub(crate) fn get_latest_compatible_asset<'a>(
    releases: &'a [Release],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<ModLoader>,
) -> Option<(&'a Asset, bool)> {
    // Combine all the assets of every release
    let assets = releases.iter().flat_map(|r| &r.assets).collect::<Vec<_>>();

    match check::github(
        // Extract just the names of the assets
        &assets.iter().map(|a| &a.name).collect::<Vec<_>>(),
        game_version_to_check,
        mod_loader_to_check,
    ) {
        Some(index) => Some((assets[index], false)),
        None => {
            if mod_loader_to_check == Some(ModLoader::Quilt) {
                get_latest_compatible_asset(
                    releases,
                    game_version_to_check,
                    Some(ModLoader::Fabric),
                )
                .map(|some| (some.0, true))
            } else {
                None
            }
        }
    }
}

/// Get the latest compatible downloadable from the `mod_` provided
///
/// Also returns whether Fabric backwards compatibility for Quilt was used.
pub async fn get_latest_compatible_downloadable(
    apis: APIs<'_>,
    mod_: &Mod,
    game_version_to_check: &str,
    mod_loader_to_check: ModLoader,
) -> Result<(Downloadable, bool)> {
    let game_version_to_check = if mod_.check_game_version {
        Some(game_version_to_check)
    } else {
        None
    };
    let mod_loader_to_check = if mod_.check_mod_loader {
        Some(mod_loader_to_check)
    } else {
        None
    };

    match &mod_.identifier {
        ModIdentifier::CurseForgeProject(project_id) => get_latest_compatible_file(
            apis.cf.get_mod_files(*project_id).await?,
            game_version_to_check,
            mod_loader_to_check,
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.try_into()?, ok.1)),
        ),
        ModIdentifier::ModrinthProject(project_id) => get_latest_compatible_version(
            &apis.mr.list_versions(project_id).await?,
            game_version_to_check,
            mod_loader_to_check,
        )
        .map_or_else(
            || Err(Error::NoCompatibleFile),
            |ok| Ok((ok.0.clone().into(), ok.2)),
        ),
        ModIdentifier::GitHubRepository(full_name) => get_latest_compatible_asset(
            &apis
                .gh
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
            |ok| Ok((ok.0.clone().into(), ok.1)),
        ),
    }
}
