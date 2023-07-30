use crate::{
    config::structs::{ModIdentifier, Profile},
    upgrade::mod_downloadable,
};
use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
#[error("{}: {}", self, .0)]
pub enum Error {
    #[error(
        "The developer of this project has denied third party applications from downloading it"
    )]
    /// The user can manually download the mod and place it in the `user` folder of the output directory to mitigate this.
    /// However, they will have to manually update the mod.
    DistributionDenied,
    #[error("The project has already been added")]
    AlreadyAdded,
    #[error("The project does not exist")]
    DoesNotExist,
    #[error("The project is not compatible")]
    Incompatible,
    #[error("The project is not a mod")]
    NotAMod,
    GitHubError(octocrab::Error),
    ModrinthError(ferinth::Error),
    CurseForgeError(furse::Error),
}
type Result<T> = std::result::Result<T, Error>;

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
        if let octocrab::Error::GitHub { source, .. } = &err {
            if &source.message == "Not Found" {
                return Self::DoesNotExist;
            }
        }
        Self::GitHubError(err)
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
        mod_.name.to_lowercase() == repo.name.to_lowercase()
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
            if should_check_game_version == Some(false) {
                None
            } else {
                Some(&profile.game_version)
            },
            if should_check_mod_loader == Some(false) {
                None
            } else {
                Some(&profile.mod_loader)
            },
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
    modrinth: &ferinth::Ferinth,
    project: &ferinth::structures::project::Project,
    profile: &Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<ferinth::structures::version::Version> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.title.to_lowercase()
            || ModIdentifier::ModrinthProject(project.id.clone()) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else if project.project_type != ferinth::structures::project::ProjectType::Mod {
        Err(Error::NotAMod)
    } else {
        let version = mod_downloadable::get_latest_compatible_version(
            &modrinth.list_versions(&project.id).await?,
            if should_check_game_version == Some(false) {
                None
            } else {
                Some(&profile.game_version)
            },
            if should_check_mod_loader == Some(false) {
                None
            } else {
                Some(&profile.mod_loader)
            },
        )
        .ok_or(Error::Incompatible)?
        .1;
        Ok(version)
    }
}

/// Check if the mod of `project_id` exists, is a mod, and is compatible with the current profile
///
/// Returns the mod and the latest compatible file
pub async fn curseforge(
    curseforge: &furse::Furse,
    project: &furse::structures::mod_structs::Mod,
    profile: &Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<furse::structures::file_structs::File> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.name.to_lowercase()
            || ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else if Some(false) == project.allow_mod_distribution {
        Err(Error::DistributionDenied)
    } else if project.links.website_url.as_str().contains("mc-mods") {
        let file = mod_downloadable::get_latest_compatible_file(
            curseforge.get_mod_files(project.id).await?,
            if should_check_game_version == Some(false) {
                None
            } else {
                Some(&profile.game_version)
            },
            if should_check_mod_loader == Some(false) {
                None
            } else {
                Some(&profile.mod_loader)
            },
        )
        .ok_or(Error::Incompatible)?
        .0;
        Ok(file)
    } else {
        Err(Error::NotAMod)
    }
}
