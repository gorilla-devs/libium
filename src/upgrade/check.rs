use crate::{config::structs::ModLoader, version_ext::VersionExt};
use ferinth::structures::version::{Version, VersionFile};
use furse::structures::file_structs::File;
use std::cmp::Reverse;

pub(crate) fn game_version_check(
    game_version_to_check: Option<&impl AsRef<str>>,
    versions: &[impl AsRef<str>],
) -> bool {
    game_version_to_check
        .map(|version| versions.iter().any(|v| v.as_ref() == version.as_ref()))
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

fn game_version_check_contain(
    game_version_to_check: Option<impl AsRef<str>>,
    versions: &[impl AsRef<str>],
) -> bool {
    game_version_to_check
        .map(|version| {
            versions
                .iter()
                .any(|v| v.as_ref().contains(version.as_ref()))
        })
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

pub(crate) fn mod_loader_check(
    mod_loader_to_check: Option<ModLoader>,
    loaders: &[impl AsRef<str>],
) -> bool {
    mod_loader_to_check
        .map(|loader| loaders.iter().any(|l| l.as_ref().parse() == Ok(loader)))
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

/// Get the latest compatible file from `files`
pub fn curseforge(
    files: &mut [File],
    game_version_to_check: Option<impl AsRef<str>>,
    mod_loader_to_check: Option<ModLoader>,
) -> Option<&File> {
    // Sort files to make the latest ones come first
    files.sort_unstable_by_key(|file1| Reverse(file1.file_date));

    // Immediately select the newest file if check is disabled, i.e. *_to_check is None
    files.iter().find(|file| {
        mod_loader_check(mod_loader_to_check, &file.game_versions)
            && game_version_check(game_version_to_check.as_ref(), &file.game_versions)
    })
}

/// Get the latest compatible version and version file from `versions`
pub fn modrinth(
    versions: &[Version],
    game_version_to_check: Option<impl AsRef<str>>,
    mod_loader_to_check: Option<ModLoader>,
) -> Option<(&VersionFile, &Version)> {
    versions
        .iter()
        .find(|version| {
            game_version_check(game_version_to_check.as_ref(), &version.game_versions)
                && mod_loader_check(mod_loader_to_check, &version.loaders)
        })
        .map(|v| (v.get_version_file(), v))
}

fn is_jar_file(asset_name: impl AsRef<str>) -> bool {
    asset_name.as_ref().ends_with(".jar")
}

fn is_not_source(asset_name: impl AsRef<str>) -> bool {
    !asset_name.as_ref().contains("source")
}

/// Search through release asset names and return the index of a compatible one
pub fn github(
    asset_names: &[impl AsRef<str>],
    game_version_to_check: Option<impl AsRef<str>>,
    mod_loader_to_check: Option<ModLoader>,
) -> Option<usize> {
    asset_names.iter().map(AsRef::as_ref).position(|name| {
        is_jar_file(name)
            && is_not_source(name)
            && game_version_check_contain(
                game_version_to_check.as_ref(),
                &name
                    .strip_suffix(".jar")
                    .unwrap()
                    .split('-')
                    .collect::<Vec<_>>(),
            )
            && mod_loader_check(
                mod_loader_to_check,
                &name
                    .strip_suffix(".jar")
                    .unwrap()
                    .split('-')
                    .collect::<Vec<_>>(),
            )
    })
}
