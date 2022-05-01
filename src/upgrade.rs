use crate::config::{self, structs::ModLoader};
use ferinth::{structures::version_structs::Version, Ferinth};
use furse::{structures::file_structs::File, Furse};
use octocrab::{models::repos::Asset, repos::RepoHandler};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not find a compatible file to download")]
    NoCompatibleFile,
    #[error("{}", .0)]
    GitHubError(#[from] octocrab::Error),
    #[error("{}", .0)]
    ModrinthError(#[from] ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(#[from] furse::Error),
    #[error("{}", .0)]
    IOError(#[from] tokio::io::Error),
}

/// Write `contents` to a file with path `profile.output_dir`/`file_name`
pub async fn write_mod_file(
    profile: &config::structs::Profile,
    contents: bytes::Bytes,
    file_name: &str,
) -> tokio::io::Result<()> {
    // Open the mod JAR file
    let mut mod_file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(profile.output_dir.join(file_name))
        .await?;

    // Write downloaded contents to mod JAR file
    mod_file.write_all(&contents).await?;
    Ok(())
}

/// Check if the target `to_check` version is present in `game_versions`.
fn check_game_version(game_versions: &[String], to_check: &str) -> bool {
    game_versions.iter().any(|version| version == to_check)
}

/// Check if the target `to_check` mod loader is present in `mod_loaders`
fn check_mod_loader(mod_loaders: &[String], to_check: &ModLoader) -> bool {
    for mod_loader in mod_loaders {
        if let Ok(mod_loader) = ModLoader::try_from(mod_loader) {
            if &mod_loader == to_check {
                return true;
            }
        }
    }
    false
}

/// Get the latest compatible file of `project_id`
///
/// Returns an additional boolean that is true if the file is supported though backwards compatibility
/// (e.g. Fabric mods running on Quilt)
pub async fn curseforge(
    curseforge: &Furse,
    profile: &config::structs::Profile,
    project_id: i32,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(File, bool)> {
    let mut files = curseforge.get_mod_files(project_id).await?;
    files.sort_unstable_by_key(|file| file.file_date);
    // Reverse so that the newest files come first
    files.reverse();

    for file in files {
        // Cancels the checks by short circuiting if it should not check
        if Some(false) == should_check_game_version
            || check_game_version(&file.game_versions, &profile.game_version)
        {
            if Some(false) == should_check_mod_loader
                || check_mod_loader(&file.game_versions, &profile.mod_loader)
            {
                return Ok((file, false));
            }
            if Some(false) == should_check_mod_loader
                || (profile.mod_loader == ModLoader::Quilt
                    && check_mod_loader(&file.game_versions, &ModLoader::Fabric))
            {
                return Ok((file, true));
            }
        }
    }
    Err(Error::NoCompatibleFile)
}

/// Download and install the latest version of `project_id`
pub async fn modrinth(
    modrinth: &Ferinth,
    profile: &config::structs::Profile,
    project_id: &str,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(Version, bool)> {
    let versions = modrinth.list_versions(project_id).await?;

    for version in versions {
        // Cancels the checks by short circuiting if it should not check
        if Some(false) == should_check_game_version
            || check_game_version(&version.game_versions, &profile.game_version)
        {
            if Some(false) == should_check_mod_loader
                || check_mod_loader(&version.loaders, &profile.mod_loader)
            {
                return Ok((version, false));
            }
            if Some(false) == should_check_mod_loader
                || (profile.mod_loader == ModLoader::Quilt
                    && check_mod_loader(&version.loaders, &ModLoader::Fabric))
            {
                return Ok((version, true));
            }
        }
    }
    Err(Error::NoCompatibleFile)
}

/// Download and install the latest release of `repo_handler`
pub async fn github(
    repo_handler: &RepoHandler<'_>,
    profile: &config::structs::Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(Asset, bool)> {
    let releases = repo_handler.releases().list().send().await?;

    for release in &releases {
        let release_name = release.name.as_ref().unwrap();
        for asset in &release.assets {
            if asset.name.contains("jar")
                // Sources JARs should not be used with the regular game
                && !asset.name.contains("sources")
                // Cancels the checks by short circuiting if it should not check
                && (Some(false) == should_check_game_version
                || asset.name.contains(&profile.game_version)
                || release_name.contains(&profile.game_version))
            {
                let asset_name = asset
                    .name
                    .split('-')
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if Some(false) == should_check_mod_loader
                    || check_mod_loader(&asset_name, &profile.mod_loader)
                {
                    return Ok((asset.clone(), false));
                }
                if Some(false) == should_check_mod_loader
                    || (profile.mod_loader == ModLoader::Quilt
                        && check_mod_loader(&asset_name, &ModLoader::Fabric))
                {
                    return Ok((asset.clone(), true));
                }
            }
        }
    }
    Err(Error::NoCompatibleFile)
}
