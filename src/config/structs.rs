use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
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
    /// The project ID of the modpack
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum ModLoader {
    Quilt,
    Fabric,
    Forge,
}
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("The given string is not a mod loader")]
pub struct ModLoaderParseError {}
impl TryFrom<&str> for ModLoader {
    type Error = ModLoaderParseError;
    fn try_from(from: &str) -> Result<Self, Self::Error> {
        match from.to_lowercase().as_str() {
            "quilt" => Ok(Self::Quilt),
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            _ => Err(Self::Error {}),
        }
    }
}
