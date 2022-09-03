use crate::{
    config::structs::{ModIdentifier, Profile},
    upgrade::mod_downloadable,
};
use reqwest::StatusCode;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The developer of this mod has denied third party applications from downloading it")]
    /// The user can manually download the mod and place it in the `user` folder of the output directory to mitigate this.
    /// However, they will have to manually update the mod
    DistributionDenied,
    #[error("The project/repository has already been added")]
    AlreadyAdded,
    #[error("The project/repository does not exist")]
    DoesNotExist,
    #[error("The project/repository is not compatible")]
    Incompatible,
    #[error("The project/repository is not a mod")]
    NotAMod,
    #[error("{}", .0)]
    GitHubError(octocrab::Error),
    #[error("{}", .0)]
    ModrinthError(ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(furse::Error),
}

impl From<furse::Error> for Error {
    fn from(err: furse::Error) -> Self {
        if let furse::Error::ReqwestError(source) = &err {
            if Some(StatusCode::NOT_FOUND) == source.status() {
                Self::DoesNotExist
            } else {
                Self::CurseForgeError(err)
            }
        } else {
            Self::CurseForgeError(err)
        }
    }
}

impl From<ferinth::Error> for Error {
    fn from(err: ferinth::Error) -> Self {
        if let ferinth::Error::ReqwestError(source) = &err {
            if Some(StatusCode::NOT_FOUND) == source.status() {
                Self::DoesNotExist
            } else {
                Self::ModrinthError(err)
            }
        } else {
            Self::ModrinthError(err)
        }
    }
}

impl From<octocrab::Error> for Error {
    fn from(err: octocrab::Error) -> Self {
        if let octocrab::Error::Http { source, .. } = &err {
            if Some(StatusCode::NOT_FOUND) == source.status() {
                Self::DoesNotExist
            } else {
                Self::GitHubError(err)
            }
        } else {
            Self::GitHubError(err)
        }
    }
}

/// Check if the repo of `repo_handler` exists, releases mods, and is compatible with the current profile
///
/// Returns the repository and the latest compatible asset
pub async fn github(
    repo_handler: &octocrab::repos::RepoHandler<'_>,
    profile: &Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(octocrab::models::Repository, octocrab::models::repos::Asset)> {
    let repo = repo_handler.get().await?;
    let repo_name = (
        repo.owner.as_ref().unwrap().login.clone(),
        repo.name.clone(),
    );

    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name == repo.name
            || ModIdentifier::GitHubRepository(repo_name.clone()) == mod_.identifier
    }) {
        return Err(Error::AlreadyAdded);
    }

    let releases = repo_handler.releases().list().send().await?.items;
    let mut contains_jar_asset = false;

    // Check if the releases contain a JAR file
    'outer: for release in &releases {
        for asset in &release.assets {
            if asset.name.contains("jar") {
                contains_jar_asset = true;
                break 'outer;
            }
        }
    }

    if contains_jar_asset {
        let asset = mod_downloadable::get_latest_compatible_asset(
            &releases,
            &profile.game_version,
            &profile.mod_loader,
            should_check_game_version,
            should_check_mod_loader,
        )
        .ok_or(Error::Incompatible)?
        .0;
        Ok((repo, asset))
    } else {
        Err(Error::NotAMod)
    }
}

/// Check if the project of `project_id` exists, is a mod, and is compatible with the current profile
///
/// Returns the project and the latest compatible version
pub async fn modrinth(
    modrinth: Arc<ferinth::Ferinth>,
    project_id: &str,
    profile: &Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(
    ferinth::structures::project_structs::Project,
    ferinth::structures::version_structs::Version,
)> {
    let project = modrinth.get_project(project_id).await?;
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name == project.title
            || ModIdentifier::ModrinthProject(project.id.clone()) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else if project.project_type != ferinth::structures::project_structs::ProjectType::Mod {
        Err(Error::NotAMod)
    } else {
        let version = mod_downloadable::get_latest_compatible_version(
            &modrinth.list_versions(&project.id).await?,
            &profile.game_version,
            &profile.mod_loader,
            should_check_game_version,
            should_check_mod_loader,
        )
        .ok_or(Error::Incompatible)?
        .1;
        Ok((project, version))
    }
}

/// Check if the mod of `project_id` exists, is a mod, and is compatible with the current profile
///
/// Returns the mod and the latest compatible file
pub async fn curseforge(
    curseforge: Arc<furse::Furse>,
    project: &furse::structures::mod_structs::Mod,
    profile: &Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<furse::structures::file_structs::File> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name == project.name || ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    }) {
        return Err(Error::AlreadyAdded);
    }

    if Some(false) == project.allow_mod_distribution {
        return Err(Error::DistributionDenied);
    }

    let files = curseforge.get_mod_files(project.id).await?;
    let mut contains_jar_file = false;

    // Check if the files are JAR files
    for file in &files {
        if file.file_name.contains("jar") {
            contains_jar_file = true;
            break;
        }
    }

    if contains_jar_file {
        let file = mod_downloadable::get_latest_compatible_file(
            files,
            &profile.game_version,
            &profile.mod_loader,
            should_check_game_version,
            should_check_mod_loader,
        )
        .ok_or(Error::Incompatible)?
        .0;
        Ok(file)
    } else {
        Err(Error::NotAMod)
    }
}
