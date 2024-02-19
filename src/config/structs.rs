use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    /// The index of the active profile
    pub active_profile: usize,
    /// The index of the active modpack
    pub active_modpack: usize,
    pub profiles: Vec<Profile>,
    pub modpacks: Vec<Modpack>,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    /// The project ID of the mod
    pub identifier: ModIdentifier,
    /// Whether to check for game version compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_game_version: Option<bool>,
    /// Whether to check for mod loader compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_mod_loader: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifier {
    CurseForgeProject(i32),
    ModrinthProject(String),
    GitHubRepository((String, String)),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifierRef<'a> {
    CurseForgeProject(i32),
    ModrinthProject(&'a str),
    GitHubRepository((&'a str, &'a str)),
}

impl ModIdentifier {
    pub fn as_ref(&self) -> ModIdentifierRef {
        match self {
            ModIdentifier::CurseForgeProject(id) => ModIdentifierRef::CurseForgeProject(*id),
            ModIdentifier::ModrinthProject(_) => todo!(),
            ModIdentifier::GitHubRepository(_) => todo!(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
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
        match from.to_lowercase().as_str() {
            "quilt" => Ok(Self::Quilt),
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            "neoforge" => Ok(Self::NeoForge),
            _ => Err(Self::Err {}),
        }
    }
}
