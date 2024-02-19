use crate::{
    config::structs::{Mod, ModIdentifier, Profile},
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

/// Check if the repo of `repo_handler` exists, releases mods,
/// and is compatible `profile`. If so, add its ID to it.
///
/// Returns the name of the repository to display to the user
pub async fn github(
    repo_handler: &octocrab::repos::RepoHandler<'_>,
    profile: &mut Profile,
    perform_checks: bool,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<String> {
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

    if perform_checks {
        let releases = repo_handler.releases().list().send().await?.items;

        if !releases
            .iter()
            .flat_map(|r| &r.assets)
            .any(|a| a.name.ends_with(".jar"))
        {
            return Err(Error::NotAMod);
        }

        mod_downloadable::get_latest_compatible_asset(
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
        .ok_or(Error::Incompatible)?;
    }

    profile.mods.push(Mod {
        name: repo.name.trim().into(),
        identifier: ModIdentifier::GitHubRepository((
            repo.owner.expect("Could not get repository owner").login,
            repo.name.clone(),
        )),
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

    Ok(repo.name)
}

use ferinth::structures::project::{DonationLink, ProjectType};
/// Check if the project of `project_id` exists, is a mod, and is compatible with `profile`.
/// If so, add its ID to `profile`.
///
/// Returns the project name and donation URLs to display to the user
pub async fn modrinth(
    modrinth: &ferinth::Ferinth,
    project_id: &str,
    profile: &mut Profile,
    perform_checks: bool,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<(String, Vec<DonationLink>)> {
    let project = modrinth.get_project(project_id).await?;
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.title.to_lowercase()
            || ModIdentifier::ModrinthProject(project.id.clone()) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else if project.project_type != ProjectType::Mod {
        Err(Error::NotAMod)
    } else {
        if perform_checks {
            mod_downloadable::get_latest_compatible_version(
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
            .ok_or(Error::Incompatible)?;
        }

        profile.mods.push(Mod {
            name: project.title.trim().into(),
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

        Ok((project.title, project.donation_urls))
    }
}

/// Check if the mod of `project_id` exists, is a mod, and is compatible with `profile`.
/// If so, add its ID to `profile`.
///
/// Returns the mod name to display to the user
pub async fn curseforge(
    curseforge: &furse::Furse,
    project_id: i32,
    profile: &mut Profile,
    perform_checks: bool,
    should_check_game_version: Option<bool>,
    should_check_mod_loader: Option<bool>,
) -> Result<String> {
    let project = curseforge.get_mod(project_id).await?;
    // Check if project has already been added

    if profile.mods.iter().any(|mod_| {
        mod_.name.to_lowercase() == project.name.to_lowercase()
            || ModIdentifier::CurseForgeProject(project.id) == mod_.identifier
    }) {
        Err(Error::AlreadyAdded)
    } else if Some(false) == project.allow_mod_distribution {
        Err(Error::DistributionDenied)
    } else if !project.links.website_url.as_str().contains("mc-mods") {
        Err(Error::NotAMod)
    } else {
        if perform_checks {
            mod_downloadable::get_latest_compatible_file(
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
            .ok_or(Error::Incompatible)?;
        }

        profile.mods.push(Mod {
            name: project.name.trim().into(),
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

        Ok(project.name)
    }
}
