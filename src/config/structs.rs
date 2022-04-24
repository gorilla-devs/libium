use clap::ArgEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    /// The index of the active profile
    pub active_profile: usize,
    /// The profiles
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    /// The profile's name
    pub name: String,
    /// The directory to download mod JARs to
    pub output_dir: PathBuf,
    /// Check if mod JARs are compatible with this Minecraft version
    pub game_version: String,
    /// Check if mod JARs are compatible with this mod loader
    pub mod_loader: ModLoader,
    /// A list of all the mods configured
    pub mods: Vec<Mod>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    /// The name the mod
    pub name: String,
    /// An enum to identify the mod based on a mod source
    pub identifier: ModIdentifier,
    /// Whether to check for game version compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_game_version: Option<bool>,
    /// Whether to check for mod loader compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_mod_loader: Option<bool>,
}

/// A mod identifier, which can be from 3 different sources
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifier {
    CurseForgeProject(i32),
    ModrinthProject(String),
    GitHubRepository((String, String)),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ArgEnum)]
pub enum ModLoader {
    Quilt,
    Fabric,
    Forge,
}

#[derive(thiserror::Error, Debug)]
#[error("The given string is not a mod loader")]
pub struct ModLoaderParseError {}

impl TryFrom<&String> for ModLoader {
    type Error = ModLoaderParseError;
    fn try_from(from: &String) -> Result<Self, Self::Error> {
        match from.to_lowercase().as_str() {
            "quilt" => Ok(Self::Quilt),
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            _ => Err(Self::Error {}),
        }
    }
}
