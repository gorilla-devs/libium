use crate::config::structs::{Mod, ModIdentifier, Profile};
use ferinth::{
    structures::project_structs::{Project, ProjectType},
    Ferinth,
};
use furse::Furse;
use octocrab::{models::Repository, repos::RepoHandler};
use reqwest::StatusCode;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Mod already added to profile")]
    AlreadyAdded,
    #[error("The provided mod does not exist")]
    DoesNotExist,
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

/// Check if the repo of `repo_handler` exists and releases mods, and if so add the repo to `profile`
pub async fn github(
    repo_handler: &RepoHandler<'_>,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<Repository> {
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
    'outer: for release in releases {
        for asset in release.assets {
            if asset.name.contains("jar") {
                contains_jar_asset = true;
                break 'outer;
            }
        }
    }

    if contains_jar_asset {
        profile.mods.push(Mod {
            name: repo.name.clone(),
            identifier: ModIdentifier::GitHubRepository(repo_name),
            check_game_version: if should_check_game_version == Some(true) {
                None
            } else {
                should_check_game_version
            },
            check_mod_loader: if should_check_mod_loader == Some(true) {
                None
            } else {
                should_check_mod_loader
            },
        });
        Ok(repo)
    } else {
        Err(Error::NotAMod)
    }
}

/// Check if the project of `project_id` exists and is a mod, if so add the project to `profile`
pub async fn modrinth(
    modrinth: Arc<Ferinth>,
    project_id: &str,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<Project> {
    let project = modrinth.get_project(project_id).await?;
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name == project.title
            || ModIdentifier::ModrinthProject(project.id.clone()) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else if project.project_type != ProjectType::Mod {
        Err(Error::NotAMod)
    } else {
        profile.mods.push(Mod {
            name: project.title.clone(),
            identifier: ModIdentifier::ModrinthProject(project.id.clone()),
            check_game_version: if should_check_game_version == Some(true) {
                None
            } else {
                should_check_game_version
            },
            check_mod_loader: if should_check_mod_loader == Some(true) {
                None
            } else {
                should_check_mod_loader
            },
        });
        Ok(project)
    }
}

/// Check if the mod of `project_id` exists, if so add that mod to `profile`
pub async fn curseforge(
    curseforge: Arc<Furse>,
    project_id: i32,
    profile: &mut Profile,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<furse::structures::mod_structs::Mod> {
    let project = curseforge.get_mod(project_id).await?;
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name == project.name || ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else {
        profile.mods.push(Mod {
            name: project.name.clone(),
            identifier: ModIdentifier::CurseForgeProject(project.id),
            check_game_version: if should_check_game_version == Some(true) {
                None
            } else {
                should_check_game_version
            },
            check_mod_loader: if should_check_mod_loader == Some(true) {
                None
            } else {
                should_check_mod_loader
            },
        });
        Ok(project)
    }
}
