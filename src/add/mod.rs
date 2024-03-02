use crate::config::structs::Profile;
use bitflags::bitflags;
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

bitflags! {
    pub struct Checks: u8 {
        const ENABLED = 0b00000001;
        const GAME_VERSION = 0b00000010;
        const MOD_LOADER = 0b00000100;
    }
}

pub struct ModProvider<'p> {
    modrinth: &'p ferinth::Ferinth,
    curseforge: &'p furse::Furse,
    github: &'p octocrab::Octocrab,
    checks: &'p Checks,
    profile: &'p mut Profile,
}

impl<'p> ModProvider<'p> {
    pub fn new(
        modrinth: &'p ferinth::Ferinth,
        curseforge: &'p furse::Furse,
        github: &'p octocrab::Octocrab,
        checks: &'p Checks,
        profile: &'p mut Profile,
    ) -> Self {
        Self {
            modrinth,
            curseforge,
            github,
            checks,
            profile,
        }
    }

    pub async fn add(&mut self, identifier: &str) -> Result<String> {
        if let Ok(project_id) = identifier.parse() {
            self.curseforge(project_id).await
        } else if identifier.matches('/').count() == 1 {
            self.github(identifier).await
        } else {
            self.modrinth(identifier).await
        }
    }

    pub async fn curseforge(&mut self, project_id: i32) -> Result<String> {
        curseforge::curseforge(self.curseforge, project_id, self.profile, self.checks).await
    }
    pub async fn github(&mut self, identifier: &str) -> Result<String> {
        let split = identifier.split('/').collect::<Vec<_>>();
        let repo_handler = self.github.repos(split[0], split[1]);
        github::github(&repo_handler, self.profile, self.checks).await
    }
    pub async fn modrinth(&mut self, identifier: &str) -> Result<String> {
        modrinth::modrinth(self.modrinth, identifier, self.profile, self.checks)
            .await
            .map(|o| o.0)
    }
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
        if let octocrab::Error::GitHub { source, .. } = &err {
            if &source.message == "Not Found" {
                return Self::DoesNotExist;
            }
        }
        Self::GitHubError(err)
    }
}

pub async fn add_multiple<'p>(
    mod_provider: &mut ModProvider<'p>,
    identifiers: Vec<String>,
) -> (Vec<String>, Vec<(String, Error)>) {
    let mut success_names = Vec::new();
    let mut failures = Vec::new();

    for identifier in identifiers {
        mod_provider
            .add(&identifier)
            .await
            .map(|name| success_names.push(name))
            .map_err(|err| {
                let ret_err =
                    if matches!(err, Error::ModrinthError(ferinth::Error::InvalidIDorSlug)) {
                        Error::InvalidIdentifier
                    } else {
                        err
                    };
                failures.push((identifier, ret_err))
            })
            .ok();
    }
    (success_names, failures)
}

#[deprecated(note = "use ModProvide::add() instead")]
pub async fn add_single(
    modrinth: &ferinth::Ferinth,
    curseforge: &furse::Furse,
    github: &octocrab::Octocrab,
    profile: &mut Profile,
    identifier: &str,
    checks: &Checks,
) -> Result<String> {
    ModProvider::new(modrinth, curseforge, github, checks, profile)
        .add(identifier)
        .await
}
