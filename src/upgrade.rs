use crate::config;
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
    CurseForgeError(#[from] reqwest::Error),
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

/// Check if the target `to_check` version is contained in `game_versions`.
fn check_version(game_versions: &Vec<String>, to_check: &str) -> bool {
    game_versions.iter().any(|version| version == &to_check)
}

/// Download and install the latest file of `project_id`
pub async fn curseforge(
    curseforge: &Furse,
    profile: &config::structs::Profile,
    project_id: i32,
    check_game_version: Option<bool>,
    check_mod_loader: Option<bool>,
) -> Result<File> {
    let mut files = curseforge.get_mod_files(project_id).await?;
    files.sort_unstable_by_key(|file| file.file_date);
    // Reverse so that the newest files come first
    files.reverse();
    let mut latest_compatible_file = None;

    for file in files {
        // Cancels the checks by short circuiting if it should not check
        if (Some(false) == check_mod_loader
            || file.game_versions.contains(&profile.mod_loader.to_string()))
            && (Some(false) == check_game_version
                || check_version(&file.game_versions, &profile.game_version))
        {
            latest_compatible_file = Some(file);
            break;
        }
    }

    latest_compatible_file.ok_or(Error::NoCompatibleFile)
}

/// Download and install the latest version of `project_id`
pub async fn modrinth(
    modrinth: &Ferinth,
    profile: &config::structs::Profile,
    project_id: &str,
    check_game_version: Option<bool>,
    check_mod_loader: Option<bool>,
) -> Result<Version> {
    let versions = modrinth.list_versions(project_id).await?;
    let mut latest_compatible_version = None;

    for version in versions {
        // Cancels the checks by short circuiting if it should not check
        if (Some(false) == check_mod_loader
            || version
                .loaders
                .contains(&profile.mod_loader.to_string().to_lowercase()))
            && (Some(false) == check_game_version
                || check_version(&version.game_versions, &profile.game_version))
        {
            latest_compatible_version = Some(version);
            break;
        }
    }

    latest_compatible_version.ok_or(Error::NoCompatibleFile)
}

/// Download and install the latest release of `repo_handler`
pub async fn github(
    repo_handler: &RepoHandler<'_>,
    profile: &config::structs::Profile,
    check_game_version: Option<bool>,
    check_mod_loader: Option<bool>,
) -> Result<Asset> {
    let releases = repo_handler.releases().list().send().await?;
    let mut asset_to_download = None;

    'outer: for release in &releases {
        let release_name = release.name.as_ref().unwrap();
        for asset in &release.assets {
            // Cancels the checks by short circuiting if it should not check
            if (Some(false) == check_mod_loader
					|| asset.name.to_lowercase().contains(&profile.mod_loader.to_string().to_lowercase()))
                    // Check if the game version is compatible
                    && (
                        Some(false) == check_game_version
                        || asset.name.contains(&profile.game_version)
                        || release_name.contains(&profile.game_version)
                    )
                    // Check if its a JAR file
                    && asset.name.contains("jar")
            {
                asset_to_download = Some(asset.clone());
                break 'outer;
            }
        }
    }

    asset_to_download.ok_or(Error::NoCompatibleFile)
}
