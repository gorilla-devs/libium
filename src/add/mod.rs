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
    /// Represents a set of boolean flags used for checking conditions while adding Mods using
    /// [ModProvider]
    ///
    /// The `Checks` struct is a bitflag representation of different checks that can be performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use libium::Checks;
    ///
    /// // Create a set of checks
    /// let checks = Checks::ENABLED | Checks::GAME_VERSION;
    /// // or
    /// let mut checks = Checks::empty();
    /// checks.insert(Checks::ENABLED);
    ///
    ///
    /// // Check if a specific flag is set
    /// if checks.contains(Checks::ENABLED) {
    ///     println!("The feature is enabled.");
    /// }
    ///
    /// // Add additional checks
    /// let updated_checks = checks | Checks::MOD_LOADER;
    /// ```
    pub struct Checks: u8 {
        /// Should we perform checks?
        const ENABLED = 0b00000001;
        /// Should we check game version?
        const GAME_VERSION = 0b00000010;
        /// Should we check mod loader?
        const MOD_LOADER = 0b00000100;
    }
}

/// Collects all mod providers (i.e Modrinth, Cursefore and github wrappers) and abstracts away the
/// add method
pub struct ModProvider<'p> {
    modrinth: &'p ferinth::Ferinth,
    curseforge: &'p furse::Furse,
    github: &'p octocrab::Octocrab,
    checks: &'p Checks,
    profile: &'p mut Profile,
}

impl<'p> ModProvider<'p> {
    /// Creates a new instance of `ModManager` with the provided API wrappers, checks, and profile.
    ///
    /// This function constructs a new `ModManager` instance, which serves as a utility for adding mods.
    /// It takes API wrappers for Modrinth, CurseForge, and GitHub, along with references to checks
    /// and a mutable reference to a profile. These components are used internally for
    /// for adding mods, performing checks, and managing profiles.
    ///
    /// # Arguments
    ///
    /// * `modrinth` - A reference to the Modrinth API wrapper (`ferinth::Ferinth`).
    /// * `curseforge` - A reference to the CurseForge API wrapper (`furse::Furse`).
    /// * `github` - A reference to the GitHub API wrapper (`octocrab::Octocrab`).
    /// * `checks` - Checks to perform while adding mods
    /// * `profile` - The profile to make changes in
    ///
    /// # Returns
    ///
    /// A new instance of `ModProvider` configured with the provided components.
    ///
    /// # Example
    ///
    /// ```
    /// // Create API wrappers
    /// let modrinth = Ferinth::new();
    /// let curseforge = Furse::new();
    /// let github = Octocrab::builder().build();
    ///
    /// // Create checks and profile
    /// let checks = Checks::empty();
    /// let mut profile = Profile::new();
    ///
    /// // Create a new ModProvider instance
    /// let mod_provider = ModProvider::new(&modrinth, &curseforge, &github, &checks, &mut profile);
    /// ```
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

    /// Add a mod to the profile based on the identifier.
    /// The identifier can be:
    ///   - A numeric ID representing a project on CurseForge.
    ///   - A GitHub repository identifier in the form "username/repository".
    ///   - Any other string, which is assumed to be a mod ID on Modrinth.
    ///
    /// # Arguments
    ///
    /// * `identifier` - A string representing the identifier of the mod.
    ///
    /// # Returns
    ///
    /// A Result containing a String representing the added mod's information,
    /// or an error if the addition failed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mod_provider = ModProvider::new(&modrinth, &curseforge, &github, &checks, &mut profile);
    /// let result = manager.add("123456");
    /// assert!(result.is_ok());
    /// ```
    pub async fn add(&mut self, identifier: &str) -> Result<String> {
        if let Ok(project_id) = identifier.parse() {
            self.curseforge(project_id).await
        } else if identifier.matches('/').count() == 1 {
            self.github(identifier).await
        } else {
            self.modrinth(identifier).await
        }
    }

    /// Fetches mod information from CurseForge using the provided project ID.
    pub async fn curseforge(&mut self, project_id: i32) -> Result<String> {
        curseforge::curseforge(self.curseforge, project_id, self.profile, self.checks).await
    }

    /// Fetches mod information from GitHub using the provided repository identifier.
    pub async fn github(&mut self, identifier: &str) -> Result<String> {
        let split = identifier.split('/').collect::<Vec<_>>();
        let repo_handler = self.github.repos(split[0], split[1]);
        github::github(&repo_handler, self.profile, self.checks).await
    }

    /// Fetches mod information from Modrinth using the provided identifier.
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

/// Adds multiple mods to the profile using the provided `ModProvider` and a list of identifiers.
///
/// # Arguments
///
/// * `mod_provider` - A mutable reference to a `ModProvider` instance used for adding mods.
/// * `identifiers` - A vector of strings representing mod identifiers to be added.
///
/// # Returns
///
/// A tuple containing two vectors:
///   - The names of the mods successfully added to the profile.
///   - Tuples of identifiers of mods that failed to be added along with the corresponding errors.
///
/// # Examples
///
/// ```
/// async fn example(mod_provider: &mut ModProvider<'_>, identifiers: Vec<String>) {
///     let (success_names, failures) = ModProvider::add_multiple(mod_provider, identifiers).await;
///
///     println!("Successfully added mods: {:?}", success_names);
///     println!("Failed to add mods:");
///     for (identifier, error) in failures {
///         println!("Identifier: {}, Error: {:?}", identifier, error);
///     }
/// }
/// ```
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
