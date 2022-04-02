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
    pub mod_loader: ModLoaders,
    /// A list of all the mods configured
    pub mods: Vec<Mod>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    /// The name the mod
    pub name: String,
    /// An enum to identify the mod based on a mod source
    pub identifier: ModIdentifier,
}

/// A mod identifier, which can be from 3 different sources
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifier {
    CurseForgeProject(i32),
    ModrinthProject(String),
    GitHubRepository((String, String)),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ArgEnum)]
pub enum ModLoaders {
    Fabric,
    Forge,
}

impl std::fmt::Display for ModLoaders {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
