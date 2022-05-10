use crate::config::structs::ModLoader;
use ferinth::structures::version_structs::{Version, VersionFile};
use furse::structures::file_structs::File;
use octorust::types::{Release, ReleaseAsset};
use std::path::Path;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

/// Write `contents` to a file with path `output_dir`/`file_name`
pub async fn write_mod_file(
    output_dir: &Path,
    contents: bytes::Bytes,
    file_name: &str,
) -> tokio::io::Result<()> {
    let mut mod_file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_dir.join(file_name))
        .await?;
    mod_file.write_all(&contents).await?;
    Ok(())
}

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
            for file in &version.files {
                if file.primary {
                    return Some((file, version));
                }
            }
            return Some((&version.files[0], version));
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
) -> Option<&'a ReleaseAsset> {
    for release in releases {
        for asset in &release.assets {
            if asset.name.contains("jar")
                // Sources JARs should not be used with the regular game
                && !asset.name.contains("sources")
                && (Some(false) == should_check_game_version
                    || asset.name.contains(game_version_to_check))
                && (Some(false) == should_check_mod_loader
                    || check_mod_loader(
                        &asset
                            .name
                            .split('-')
                            .map(str::to_string)
                            .collect::<Vec<_>>(),
                        mod_loader_to_check,
                    ))
            {
                return Some(asset);
            }
        }
    }
    None
}
