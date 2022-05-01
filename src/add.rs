use crate::{
    config,
    config::structs::{Mod, ModIdentifier},
};
use ferinth::{
    structures::project_structs::{Project, ProjectType},
    Ferinth,
};
use furse::Furse;
use octocrab::{models::Repository, repos::RepoHandler};
use reqwest::StatusCode;

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

/// Check if repo of `repo_handler` exists and releases mods, and if so add the repo to `profile`
pub async fn github(
    repo_handler: RepoHandler<'_>,
    profile: &mut config::structs::Profile,
) -> Result<Repository> {
    let repo = repo_handler.get().await?;
    // Get the name of the repository as a tuple
    let repo_name_split = repo
        .full_name
        .as_ref()
        .unwrap()
        .split('/')
        .collect::<Vec<_>>();
    let repo_name = (repo_name_split[0].into(), repo_name_split[1].into());

    if profile.mods.iter().any(|mod_| {
        config::structs::ModIdentifier::GitHubRepository(repo_name.clone()) == mod_.identifier
    }) {
        return Err(Error::AlreadyAdded);
    }

    let releases = repo_handler.releases().list().send().await?;
    let mut contains_jar_asset = false;

    // Search every asset to check if the releases contain JAR files (a mod file)
    'outer: for release in releases {
        for asset in release.assets {
            if asset.name.contains("jar") {
                // If JAR release is found, set flag to true and break
                contains_jar_asset = true;
                break 'outer;
            }
        }
    }

    if contains_jar_asset {
        profile.mods.push(Mod {
            name: repo.name.clone(),
            identifier: ModIdentifier::GitHubRepository(repo_name),
            check_game_version: None,
            check_mod_loader: None,
        });
        Ok(repo)
    } else {
        Err(Error::NotAMod)
    }
}

/// Check if `project_id` exists and is a mod, if so add that project ID to `profile`
/// Returns the project struct
pub async fn modrinth(
    modrinth: &Ferinth,
    project_id: &str,
    profile: &mut config::structs::Profile,
) -> Result<Project> {
    let project = modrinth.get_project(project_id).await?;
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        config::structs::ModIdentifier::ModrinthProject(project.id.clone()) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    // Check that the project is a mod
    } else if project.project_type != ProjectType::Mod {
        Err(Error::NotAMod)
    } else {
        profile.mods.push(Mod {
            name: project.title.clone(),
            identifier: ModIdentifier::ModrinthProject(project.id.clone()),
            check_game_version: None,
            check_mod_loader: None,
        });
        Ok(project)
    }
}

/// Check if `project_id` exists, if so add that mod to `profile`
/// Returns the mod struct
pub async fn curseforge(
    curseforge: &Furse,
    project_id: i32,
    profile: &mut config::structs::Profile,
) -> Result<furse::structures::mod_structs::Mod> {
    let project = curseforge.get_mod(project_id).await?;
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        config::structs::ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else {
        profile.mods.push(Mod {
            name: project.name.clone(),
            identifier: ModIdentifier::CurseForgeProject(project.id),
            check_game_version: None,
            check_mod_loader: None,
        });
        Ok(project)
    }
}
