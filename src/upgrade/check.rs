use crate::{config::structs::ModLoader, version_ext::VersionExt};
use ferinth::structures::version::{Version, VersionFile};
use furse::structures::file_structs::File;
use octocrab::models::repos::{Asset, Release};

/// Get the latest compatible file from `files`
pub fn curseforge<'a>(
    files: &'a mut [File],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<&'a File> {
    // Make the newest files come first
    files.sort_unstable_by_key(|file| file.file_date);
    files.reverse();

    // Immediately select the newest file if check is disabled, i.e. *_to_check is None
    files.iter().find(|file| {
        file.game_versions.iter().any(|version| {
            game_version_to_check.is_none() || version == game_version_to_check.unwrap()
        }) && file.game_versions.iter().any(|mod_loader| {
            mod_loader_to_check.is_none()
                || Ok(mod_loader_to_check.unwrap()) == mod_loader.parse().as_ref()
        })
    })
}

/// Get the latest compatible version and version file from `versions`
pub fn modrinth<'a>(
    versions: &'a [Version],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<(&'a VersionFile, &'a Version)> {
    versions
        .iter()
        .find(|version| {
            // Immediately select the newest file if check is disabled, i.e. *_to_check is None
            version.game_versions.iter().any(|version| {
                game_version_to_check.is_none() || version == game_version_to_check.unwrap()
            }) && version.loaders.iter().any(|mod_loader| {
                mod_loader_to_check.is_none()
                    || Ok(mod_loader_to_check.unwrap()) == mod_loader.parse().as_ref()
            })
        })
        .map(|v| (v.get_version_file(), v))
}

fn is_jar_file(asset_name: &str) -> bool {
    asset_name.contains("jar")
}

fn is_not_source(asset_name: &str) -> bool {
    !asset_name.contains("source")
}

fn game_version_check(game_version: Option<&str>, asset_name: &str) -> bool {
    game_version
        .map(|game_version| asset_name.contains(game_version))
        // select latest asset if version check is disabled
        .unwrap_or(true)
}

fn mod_loader_check(mod_loader: Option<&ModLoader>, asset_name: &str) -> bool {
    mod_loader
        .map(|mod_loader| {
            asset_name
                .split('-')
                .any(|loader| loader == mod_loader.to_string().as_str())
        })
        // select latest asset if mod loader check is disabled
        .unwrap_or(true)
}

/// Get the latest compatible asset from `releases`
pub fn github<'a>(
    releases: &'a [Release],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<&'a Asset> {
    releases
        .iter()
        .flat_map(|release| {
            release
                .assets
                .iter()
                .filter(|asset| is_jar_file(&asset.name))
                .filter(|asset| is_not_source(&asset.name))
                .filter(|asset| game_version_check(game_version_to_check, &asset.name))
                .filter(|asset| mod_loader_check(mod_loader_to_check, &asset.name))
        })
        .next()
}
