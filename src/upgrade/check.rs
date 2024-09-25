use crate::config::structs::ModLoader;

use super::DownloadFile;

pub(crate) fn game_version_check(
    game_version_to_check: Option<&impl AsRef<str>>,
    versions: &[impl AsRef<str>],
) -> bool {
    game_version_to_check
        .map(|version| {
            versions
                .iter()
                .any(|v| v.as_ref().trim_start_matches("mc") == version.as_ref())
        })
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

pub(crate) fn mod_loader_check(
    mod_loader_to_check: Option<ModLoader>,
    loaders: &[ModLoader],
) -> bool {
    mod_loader_to_check
        .map(|loader| loaders.contains(&loader))
        // assume test passed if mod loader check is disabled
        .unwrap_or(true)
}

fn is_jar_file(asset_name: impl AsRef<str>) -> bool {
    asset_name.as_ref().ends_with(".jar")
}

fn is_not_source(asset_name: impl AsRef<str>) -> bool {
    !asset_name.as_ref().contains("source")
}

pub fn select_latest(
    download_files: Vec<DownloadFile>,
    game_version_to_check: Option<impl AsRef<str>>,
    mod_loader_to_check: Option<ModLoader>,
) -> Option<DownloadFile> {
    download_files.into_iter().find(|file| {
        is_jar_file(file.filename())
            && is_not_source(file.filename())
            && mod_loader_check(mod_loader_to_check, &file.loaders)
            && game_version_check(game_version_to_check.as_ref(), &file.game_versions)
    })
}
