use crate::{config, config::structs::Mod};
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
    CurseForgeError(reqwest::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if Some(StatusCode::NOT_FOUND) == err.status() {
            Self::DoesNotExist
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

    // Check if repo has already been added
    for mod_ in &profile.mods {
        if let Mod::GitHubRepository { full_name, .. } = mod_ {
            if full_name == &repo_name {
                return Err(Error::AlreadyAdded);
            }
        }
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
        profile.mods.push(Mod::GitHubRepository {
            name: repo.name.clone(),
            full_name: repo_name,
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
    project_id: String,
    profile: &mut config::structs::Profile,
) -> Result<Project> {
    let project = modrinth.get_project(&project_id).await?;
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        if let Mod::ModrinthProject { project_id, .. } = mod_ {
            project_id == &project.id
        } else {
            false
        }
    }) {
        Err(Error::AlreadyAdded)
    // Check that the project is a mod
    } else if project.project_type != ProjectType::Mod {
        Err(Error::NotAMod)
    } else {
        profile.mods.push(Mod::ModrinthProject {
            name: project.title.clone(),
            project_id: project.id.clone(),
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
        if let Mod::CurseForgeProject { project_id, .. } = mod_ {
            *project_id == project.id
        } else {
            false
        }
    }) {
        Err(Error::AlreadyAdded)
    } else {
        profile.mods.push(Mod::CurseForgeProject {
            name: project.name.clone(),
            project_id: project.id,
        });
        Ok(project)
    }
}
