use crate::{config, misc};
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
    SemVerError(#[from] semver::Error),
    #[error("{}", .0)]
    IOError(#[from] tokio::io::Error),
}

/// Write `contents` to a file with path `profile.output_dir`/`file_name`
async fn write_mod_file(
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

/// Download and install the latest file of `project_id`
///
/// If the profile has configured version `1.18.2` and if `no_patch_check` is false, the mods will be checked specifically for `1.18.2`.
/// If `no_patch_check` is true, the mods will be checked for `1.18`, `1.18.1`, `1.18.2`, or any future version of 1.18
pub async fn curseforge(
    curseforge: &Furse,
    profile: &config::structs::Profile,
    project_id: i32,
    no_patch_check: bool,
) -> Result<File> {
    let mut files = curseforge.get_mod_files(project_id).await?;
    files.sort_unstable_by_key(|file| file.file_date);
    // Reverse so that the newest files come first
    files.reverse();

    let mut latest_compatible_file = None;
    let short_game_version = misc::remove_semver_patch(&profile.game_version)?;

    for file in files {
        if no_patch_check {
            if file
                .game_versions
                .iter()
                .any(|game_version| game_version.contains(&short_game_version))
                && file.game_versions.contains(&profile.mod_loader.to_string())
            {
                latest_compatible_file = Some(file);
                break;
            }
        } else {
            // Or else just check if it contains the full version
            if file.game_versions.contains(&profile.game_version)
                && file.game_versions.contains(&profile.mod_loader.to_string())
            {
                latest_compatible_file = Some(file);
                break;
            }
        }
    }

    if let Some(file) = latest_compatible_file {
        let contents = curseforge.download_mod_file_from_file(&file).await?;
        write_mod_file(profile, contents, &file.file_name).await?;
        Ok(file)
    } else {
        Err(Error::NoCompatibleFile)
    }
}

/// Download and install the latest release of `repo_handler`
pub async fn github(
    repo_handler: &RepoHandler<'_>,
    profile: &config::structs::Profile,
) -> Result<Asset> {
    let releases = repo_handler.releases().list().send().await?;
    let version_to_check = misc::remove_semver_patch(&profile.game_version)?;

    let mut asset_to_download = None;
    // Whether the mod loader is specified in asset names
    let mut specifies_loader = false;

    'outer: for release in &releases {
        for asset in &release.assets {
            // If the asset specifies the mod loader, set the `specifies_loader` flag to true
            // If it was already set, this is skipped
            if !specifies_loader && asset.name.to_lowercase().contains("fabric")
                || asset.name.to_lowercase().contains("forge")
            {
                specifies_loader = true;
            }

            // If the mod loader is not specified then skip checking for the mod loader
            if (!specifies_loader
					// If it does specify, then check the mod loader
					|| asset.name.to_lowercase().contains(&profile.mod_loader.to_string().to_lowercase()))
                    // Check if the game version is compatible
                    && (
                        // Check the asset's name
                        asset.name.contains(&version_to_check)
						// and the release name
                        || release.name.as_ref().unwrap().contains(&version_to_check))
                    // Check if its a JAR file
                    && asset.name.contains("jar")
            {
                // Specify this asset as a compatible asset
                asset_to_download = Some(asset);
                break 'outer;
            }
        }
    }

    if let Some(asset) = asset_to_download {
        let contents = reqwest::get(asset.browser_download_url.clone())
            .await?
            .bytes()
            .await?;
        write_mod_file(profile, contents, &asset.name).await?;
        Ok(asset.clone())
    } else {
        Err(Error::NoCompatibleFile)
    }
}

/// Download and install the latest version of `project_id`
///
/// If the profile has configured version `1.18.2` and if `no_patch_check` is false, the mods will be checked specifically for `1.18.2`.
/// If `no_patch_check` is true, the mods will be checked for `1.18`, `1.18.1`, `1.18.2`, or any future version of 1.18
pub async fn modrinth(
    modrinth: &Ferinth,
    profile: &config::structs::Profile,
    project_id: &str,
    no_patch_check: bool,
) -> Result<Version> {
    let versions = modrinth.list_versions(project_id).await?;

    let mut latest_compatible_version = None;
    let short_game_version = misc::remove_semver_patch(&profile.game_version)?;

    for version in versions {
        if no_patch_check {
            // Search every version to see if it contains the short_game_version
            if version
                .game_versions
                .iter()
                .any(|game_version| game_version.contains(&short_game_version))
                && version
                    .loaders
                    .contains(&profile.mod_loader.to_string().to_lowercase())
            {
                latest_compatible_version = Some(version);
                break;
            }
        } else {
            // Or else just check if it contains the full version
            if version.game_versions.contains(&profile.game_version)
                && version
                    .loaders
                    .contains(&profile.mod_loader.to_string().to_lowercase())
            {
                latest_compatible_version = Some(version);
                break;
            }
        }
    }

    if let Some(version) = latest_compatible_version {
        let contents = modrinth.download_version_file(&version.files[0]).await?;
        write_mod_file(profile, contents, &version.files[0].filename).await?;
        Ok(version)
    } else {
        Err(Error::NoCompatibleFile)
    }
}
