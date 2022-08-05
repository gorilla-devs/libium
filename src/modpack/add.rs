use crate::config::structs::{Config, ModpackIdentifier};
use ferinth::{
    structures::project_structs::{Project, ProjectType},
    Ferinth,
};
use furse::{structures::mod_structs::Mod, Furse};
use reqwest::StatusCode;
use std::{sync::Arc};

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Modpack is already added to profile")]
    AlreadyAdded,
    #[error("The provided modpack does not exist")]
    DoesNotExist,
    #[error("The project is not a modpack")]
    NotAModpack,
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

/// Check if the project of `project_id` exists and is a modpack
///
/// Returns the project struct
pub async fn curseforge(
    curseforge: Arc<Furse>,
    config: &mut Config,
    project_id: i32,
) -> Result<Mod> {
    let project = curseforge.get_mod(project_id).await?;
    // Check if project has already been added
    if config.modpacks.iter().any(|modpack| {
        modpack.name == project.name
            || ModpackIdentifier::CurseForgeModpack(project.id) == modpack.identifier
    }) {
        return Err(Error::AlreadyAdded);
    }

    let files = curseforge.get_mod_files(project.id).await?;
    let mut contains_zip_file = false;

    // Check if the files are zip files
    for file in &files {
        if file.file_name.contains("zip") {
            contains_zip_file = true;
            break;
        }
    }

    if contains_zip_file {
        Ok(project)
    } else {
        Err(Error::NotAModpack)
    }
}

/// Check if the project of `project_id` exists and is a modpack
///
/// Returns the project struct
pub async fn modrinth(
    modrinth: Arc<Ferinth>,
    config: &mut Config,
    project_id: &str,
) -> Result<Project> {
    let project = modrinth.get_project(project_id).await?;
    // Check if project has already been added
    if config.modpacks.iter().any(|modpack| {
        modpack.name == project.title
            || ModpackIdentifier::ModrinthModpack(project.id.clone()) == modpack.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else if project.project_type != ProjectType::Modpack {
        Err(Error::NotAModpack)
    } else {
        Ok(project)
    }
}
