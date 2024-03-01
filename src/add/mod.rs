use std::cell::Cell;

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

/// Single sturct to condense check flags for game version, mod loader and to-check
/// Saves space, reduce complexity in fn args and is fast
///
/// Bit mappings (LTR: [7,6,5,4,3,2,1,0]):
/// 0: flag for "perform checks"
/// 1: flag for "game version"
/// 2: flag for "mod loader"
#[derive(Default)]
pub struct Checks(Cell<u8>);

impl Checks {
    /// Generates new [Checks] will all values set to [true]
    pub fn new_all_set() -> Self {
        Self(Cell::new(0b00000111))
    }

    /// Generates [Checks] from given predicate
    pub fn from(checks: bool, game_version: bool, mod_loader: bool) -> Self {
        let ret = Self::default();
        if checks {
            ret.set_perform_check();
        }
        if game_version {
            ret.set_game_version();
        }
        if mod_loader {
            ret.set_mod_loader();
        }
        ret
    }

    /// Set "perform_checks" bit to true
    pub fn set_perform_check(&self) {
        self.0.set(self.0.get() | 1 << 0);
    }

    /// Set "game_version" bit to true
    pub fn set_game_version(&self) {
        self.0.set(self.0.get() | 1 << 1);
    }

    /// Set "mod_loader" bit to true
    pub fn set_mod_loader(&self) {
        self.0.set(self.0.get() | 1 << 2);
    }

    /// Set "perform_checks" bit to false
    pub fn unset_perform_check(&self) {
        self.0.set(self.0.get() & 1 << 0);
    }

    /// Set "game_version" bit to false
    pub fn unset_game_version(&self) {
        self.0.set(self.0.get() & 1 << 1);
    }

    /// Set "mod_loader" bit to true
    pub fn unset_mod_loader(&self) {
        self.0.set(self.0.get() & 1 << 2);
    }

    /// Return "perform_checks" bit status
    pub fn perform_checks(&self) -> bool {
        self.0.get() & 1 != 0
    }

    /// Return "game_version" bit status
    pub fn game_version(&self) -> bool {
        self.0.get() & (1 << 1) != 0
    }

    /// Return "mod_loader" bit status
    pub fn mod_loader(&self) -> bool {
        self.0.get() & (1 << 2) != 0
    }

    /// Reset all bits to 0 (all flags to false)
    pub fn reset(&self) {
        self.0.set(0);
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
            &Checks::new_all_set(),
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

pub async fn add_single(
    modrinth: &ferinth::Ferinth,
    curseforge: &furse::Furse,
    github: &octocrab::Octocrab,
    profile: &mut Profile,
    identifier: &str,
    checks: &Checks,
) -> Result<String> {
    if let Ok(project_id) = identifier.parse() {
        curseforge::curseforge(curseforge, project_id, profile, checks).await
    } else if identifier.matches('/').count() == 1 {
        let split = identifier.split('/').collect::<Vec<_>>();
        github::github(
            &github.repos(split[0], split[1]),
            profile,
            checks.perform_checks(),
            checks,
        )
        .await
    } else {
        modrinth::modrinth(modrinth, identifier, profile, checks)
            .await
            .map(|o| o.0)
    }
}

#[cfg(test)]
mod test {
    use super::Checks;

    #[test]
    fn check_bit_set_unset() {
        let check = Checks::default();

        // seting bits
        check.set_perform_check();
        check.set_mod_loader();
        check.set_game_version();

        assert!(check.perform_checks() && check.game_version() && check.mod_loader());

        // Unset after set
        check.unset_perform_check();
        check.unset_mod_loader();
        check.unset_game_version();

        assert!(!(check.perform_checks() && check.game_version() && check.mod_loader()));

        // Unset after Unset
        check.unset_mod_loader();

        assert!(!check.mod_loader());

        // set after set
        check.set_game_version();
        check.set_game_version();

        assert!(check.game_version());
    }
}
