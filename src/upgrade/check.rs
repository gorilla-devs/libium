use crate::{config::structs::ModLoader, version_ext::VersionExt};
use ferinth::structures::version_structs::{Version, VersionFile};
use furse::structures::file_structs::File;
use octocrab::models::repos::{Asset, Release};

/// Check if the target `to_check` version is present in `game_versions`.
fn check_game_version(game_versions: &[String], to_check: &str) -> bool {
    game_versions.iter().any(|version| version == to_check)
}

/// Check if the target `to_check` mod loader is present in `mod_loaders`
fn check_mod_loader(mod_loaders: &[String], to_check: &ModLoader) -> bool {
    mod_loaders
        .iter()
        .any(|mod_loader| Ok(to_check) == ModLoader::try_from(mod_loader).as_ref())
}

/// Get the latest compatible file from `files`
pub fn curseforge<'a>(
    files: &'a mut Vec<File>,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Option<&'a File> {
    // Make the newest files come first
    files.sort_unstable_by_key(|file| file.file_date);
    files.reverse();

    for file in files {
        if (Some(false) == should_check_game_version
            || check_game_version(&file.game_versions, game_version_to_check))
            && (Some(false) == should_check_mod_loader
                || check_mod_loader(&file.game_versions, mod_loader_to_check))
        {
            return Some(file);
        }
    }
    None
}

/// Get the latest compatible version and version file from `versions`
pub fn modrinth<'a>(
    versions: &'a [Version],
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Option<(&'a VersionFile, &'a Version)> {
    for version in versions {
        if (Some(false) == should_check_game_version
            || check_game_version(&version.game_versions, game_version_to_check))
            && (Some(false) == should_check_mod_loader
                || check_mod_loader(&version.loaders, mod_loader_to_check))
        {
            return Some((version.get_version_file(), version));
        }
    }
    None
}

/// Get the latest compatible asset from `releases`
pub fn github<'a>(
    releases: &'a [Release],
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Option<&'a Asset> {
    for release in releases {
        let candidate_assets: Vec<&Asset> = release
            .assets
            .iter()
            .filter(|asset| asset.name.contains("jar") && !asset.name.contains("sources"))
            .collect();

        if let Some(asset) = candidate_assets
            .iter()
            .filter(|asset| {
                github_name_check_heuristic(
                    &asset.name,
                    game_version_to_check,
                    mod_loader_to_check,
                    should_check_game_version,
                    should_check_mod_loader,
                )
            })
            .next()
        {
            return Some(asset);
        }

        if let &[asset] = candidate_assets.as_slice() {
            if github_name_check_heuristic(
                release
                    .name
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .to_lowercase()
                    .as_str(),
                game_version_to_check,
                mod_loader_to_check,
                should_check_game_version,
                should_check_mod_loader,
            ) {
                return Some(asset);
            }
        }
    }
    None
}

fn github_name_check_heuristic<'a>(
    name: &str,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> bool {
    return (Some(false) == should_check_game_version || name.contains(game_version_to_check))
        && (Some(false) == should_check_mod_loader
            || check_mod_loader(
                &name.split('-').map(str::to_string).collect::<Vec<_>>(),
                mod_loader_to_check,
            ));
}
