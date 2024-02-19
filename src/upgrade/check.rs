use std::cmp::Reverse;

use crate::{config::structs::ModLoader, version_ext::VersionExt};
use ferinth::structures::version::{Version, VersionFile};
use furse::structures::file_structs::File;
use octocrab::models::repos::{Asset, Release};

fn game_version_check<S: AsRef<str>>(game_version_to_check: Option<&str>, versions: &[S]) -> bool {
    game_version_to_check
        .map(|version| versions.iter().any(|v| v.as_ref() == version))
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

fn game_version_check_contain<S: AsRef<str>>(
    game_version_to_check: Option<&str>,
    versions: &[S],
) -> bool {
    game_version_to_check
        .map(|version| versions.iter().any(|v| v.as_ref().contains(version)))
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

fn mod_loader_check<S: AsRef<str>>(mod_loader_to_check: Option<&ModLoader>, loaders: &[S]) -> bool {
    mod_loader_to_check
        .map(|loader| {
            loaders
                .iter()
                .any(|l| l.as_ref().parse().as_ref() == Ok(loader))
        })
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

/// Get the latest compatible file from `files`
pub fn curseforge<'a>(
    files: &'a mut [File],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<&'a File> {
    // Sort files to make the latest files come first
    files.sort_unstable_by_key(|file1| Reverse(file1.file_date));

    // Immediately select the newest file if check is disabled, i.e. *_to_check is None
    files.iter().find(|file| {
        mod_loader_check(mod_loader_to_check, &file.game_versions)
            && game_version_check(game_version_to_check, &file.game_versions)
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
            game_version_check(game_version_to_check, &version.game_versions)
                && mod_loader_check(mod_loader_to_check, &version.loaders)
        })
        .map(|v| (v.get_version_file(), v))
}

fn is_jar_file(asset_name: &str) -> bool {
    asset_name.ends_with(".jar")
}

fn is_not_source(asset_name: &str) -> bool {
    !asset_name.contains("source")
}

/// Get the latest compatible asset from `releases`
pub fn github<'a>(
    releases: &'a [Release],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<&'a Asset> {
    releases
        .iter()
        .flat_map(|release| &release.assets)
        .find(|asset| {
            is_jar_file(&asset.name)
                && is_not_source(&asset.name)
                && game_version_check_contain(
                    game_version_to_check,
                    &asset
                        .name
                        .strip_suffix(".jar")
                        .unwrap()
                        .split('-')
                        .collect::<Vec<_>>(),
                )
                && mod_loader_check(
                    mod_loader_to_check,
                    &asset
                        .name
                        .strip_suffix(".jar")
                        .unwrap()
                        .split('-')
                        .collect::<Vec<_>>(),
                )
        })
}
