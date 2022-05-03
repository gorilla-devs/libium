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
    mod_loaders
        .iter()
        .any(|mod_loader| Ok(to_check) == ModLoader::try_from(mod_loader).as_ref())
}

/// Get the latest compatible file of `project_id`
pub async fn curseforge(
    curseforge: &Furse,
    project_id: i32,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<File> {
    let mut files = curseforge.get_mod_files(project_id).await?;
    files.sort_unstable_by_key(|file| file.file_date);
    // Reverse so that the newest files come first
    files.reverse();

    for file in files {
        // Cancels the checks by short circuiting if it should not check
        if (Some(false) == should_check_game_version
            || check_game_version(&file.game_versions, game_version_to_check))
            && (Some(false) == should_check_mod_loader
                || check_mod_loader(&file.game_versions, mod_loader_to_check))
        {
            return Ok(file);
        }
    }
    Err(Error::NoCompatibleFile)
}

/// Get the latest compatible file of `project_id`
pub async fn modrinth(
    modrinth: &Ferinth,
    project_id: &str,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<Version> {
    let versions = modrinth.list_versions(project_id).await?;

    for version in versions {
        // Cancels the checks by short circuiting if it should not check
        if (Some(false) == should_check_game_version
            || check_game_version(&version.game_versions, game_version_to_check))
            && (Some(false) == should_check_mod_loader
                || check_mod_loader(&version.loaders, mod_loader_to_check))
        {
            return Ok(version);
        }
    }
    Err(Error::NoCompatibleFile)
}

/// Get the latest compatible asset of `repo_handler`
pub async fn github(
    repo_handler: &RepoHandler<'_>,
    game_version_to_check: &str,
    mod_loader_to_check: &ModLoader,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<Asset> {
    let releases = repo_handler.releases().list().send().await?;
    for release in releases {
        let release_name = release.name.as_ref().unwrap();
        for asset in release.assets {
            if asset.name.contains("jar")
                // Sources JARs should not be used with the regular game
                && !asset.name.contains("sources")
                // Cancels the checks by short circuiting if it should not check
                && (Some(false) == should_check_game_version
                || asset.name.contains(game_version_to_check)
                || release_name.contains(game_version_to_check))
                && (Some(false) == should_check_mod_loader
                || check_mod_loader(&asset
                    .name
                    .split('-')
                    .map(str::to_string)
                    .collect::<Vec<_>>(), mod_loader_to_check))
            {
                return Ok(asset);
            }
        }
    }
    Err(Error::NoCompatibleFile)
}
