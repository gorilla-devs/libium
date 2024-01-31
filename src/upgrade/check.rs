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

/// Get the latest compatible asset from `releases`
pub fn github<'a>(
    releases: &'a [Release],
    game_version_to_check: Option<&str>,
    mod_loader_to_check: Option<&ModLoader>,
) -> Option<&'a Asset> {
    for release in releases {
        for asset in &release.assets {
            if asset.name.ends_with(".jar")
                // Sources JARs should not be used with the regular game
                && !asset.name.contains("sources")
                // Immediately select the newest file if check is disabled, i.e. *_to_check is None
                && (game_version_to_check.is_none()
                    || asset.name.contains(game_version_to_check.unwrap()))
                && (mod_loader_to_check.is_none()
                    || asset.name
                        .strip_suffix(".jar")
                        .unwrap()
                        .split('-')
                        .any(|mod_loader|
                            Ok(mod_loader_to_check.unwrap()) == mod_loader.parse().as_ref()
                        ))
            {
                return Some(asset);
            }
        }
    }
    None
}
