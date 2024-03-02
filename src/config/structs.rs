use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

use crate::add::Checks;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    /// The index of the active profile
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_profile: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub profiles: Vec<Profile>,

    /// The index of the active modpack
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_modpack: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub modpacks: Vec<Modpack>,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {
    pub name: String,
    /// The Minecraft instance directory to install to
    pub output_dir: PathBuf,
    pub install_overrides: bool,
    pub identifier: ModpackIdentifier,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModpackIdentifier {
    CurseForgeModpack(i32),
    ModrinthModpack(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    /// The directory to download mod files to
    pub output_dir: PathBuf,
    /// Only download mod files compatible with this Minecraft version
    pub game_version: String,
    /// Only download mod files compatible with this mod loader
    pub mod_loader: ModLoader,
    pub mods: Vec<Mod>,
}

impl Profile {
    // Return the profile's `game_version` in `Some` if `check_game_version` is true
    pub fn get_version(&self, check_game_version: bool) -> Option<&str> {
        if check_game_version {
            Some(&self.game_version)
        } else {
            None
        }
    }

    // Return the profile's `mod_loader` in a `Some` only if `check_mod_loader` is true
    pub fn get_loader(&self, check_mod_loader: bool) -> Option<ModLoader> {
        if check_mod_loader {
            Some(self.mod_loader)
        } else {
            None
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    /// The project ID of the mod
    pub identifier: ModIdentifier,

    /// Whether to check for game version compatibility
    #[serde(skip_serializing_if = "is_true")]
    #[serde(default = "get_true")]
    pub check_game_version: bool,

    /// Whether to check for mod loader compatibility
    #[serde(skip_serializing_if = "is_true")]
    #[serde(default = "get_true")]
    pub check_mod_loader: bool,
}

impl Mod {
    pub fn new(name: &str, identifier: ModIdentifier, checks: &Checks) -> Self {
        Self {
            name: name.into(),
            identifier,
            check_game_version: checks.contains(Checks::GAME_VERSION),
            check_mod_loader: checks.contains(Checks::MOD_LOADER),
        }
    }
}

fn is_true(b: &bool) -> bool {
    *b
}

fn get_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifier {
    CurseForgeProject(i32),
    ModrinthProject(String),
    GitHubRepository((String, String)),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifierRef<'a> {
    CurseForgeProject(&'a i32),
    ModrinthProject(&'a str),
    GitHubRepository(&'a (String, String)),
}

impl ModIdentifier {
    pub fn as_ref(&self) -> ModIdentifierRef {
        match self {
            ModIdentifier::CurseForgeProject(v) => ModIdentifierRef::CurseForgeProject(v),
            ModIdentifier::ModrinthProject(v) => ModIdentifierRef::ModrinthProject(v),
            ModIdentifier::GitHubRepository(v) => ModIdentifierRef::GitHubRepository(v),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ModLoader {
    Quilt,
    Fabric,
    Forge,
    NeoForge,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("The given string is not a mod loader")]
pub struct ModLoaderParseError;

impl FromStr for ModLoader {
    type Err = ModLoaderParseError;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
        match from.trim().to_lowercase().as_str() {
            "quilt" => Ok(Self::Quilt),
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            "neoforge" => Ok(Self::NeoForge),
            _ => Err(Self::Err {}),
        }
    }
}
