use crate::config::structs::Profile;
use reqwest::StatusCode;

pub mod curseforge;
pub mod github;
pub mod modrinth;

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
    #[error("Invalid identifier")]
    InvalidIdentifier,
    GitHubError(octocrab::Error),
    ModrinthError(ferinth::Error),
    CurseForgeError(furse::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

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

pub async fn add_multiple(
    modrinth: &ferinth::Ferinth,
    curseforge: &furse::Furse,
    github: &octocrab::Octocrab,
    profile: &mut Profile,
    identifiers: Vec<String>,
) -> (Vec<String>, Vec<(String, Error)>) {
    let mut success_names = Vec::new();
    let mut failures = Vec::new();

    for identifier in identifiers {
        match add_single(
            modrinth,
            curseforge,
            github,
            profile,
            &identifier,
            true,
            true,
            true,
        )
        .await
        {
            Ok(name) => success_names.push(name),
            Err(err) => failures.push((
                identifier,
                if matches!(err, Error::ModrinthError(ferinth::Error::InvalidIDorSlug)) {
                    Error::InvalidIdentifier
                } else {
                    err
                },
            )),
        }
    }
    (success_names, failures)
}

#[allow(clippy::too_many_arguments)]
pub async fn add_single(
    modrinth: &ferinth::Ferinth,
    curseforge: &furse::Furse,
    github: &octocrab::Octocrab,
    profile: &mut Profile,
    identifier: &str,
    perform_checks: bool,
    check_game_version: bool,
    check_mod_loader: bool,
) -> Result<String> {
    if let Ok(project_id) = identifier.parse() {
        curseforge::curseforge(
            curseforge,
            project_id,
            profile,
            perform_checks,
            check_game_version,
            check_mod_loader,
        )
        .await
    } else if identifier.matches('/').count() == 1 {
        let split = identifier.split('/').collect::<Vec<_>>();
        github::github(
            &github.repos(split[0], split[1]),
            profile,
            perform_checks,
            check_game_version,
            check_mod_loader,
        )
        .await
    } else {
        modrinth::modrinth(
            modrinth,
            identifier,
            profile,
            perform_checks,
            check_game_version,
            check_mod_loader,
        )
        .await
        .map(|o| o.0)
    }
}
